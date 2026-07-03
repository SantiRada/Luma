#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lopdf::{Dictionary, Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::{
  collections::{BTreeMap, BTreeSet, HashMap},
  fs,
  fs::OpenOptions,
  fs::File,
  io::{Read, Write},
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex, OnceLock,
  },
  thread,
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tauri::{
  menu::{Menu, MenuItem, PredefinedMenuItem},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Url, WebviewUrl,
  WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{
  Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartManagerExt};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionSource {
  kind: String,
  path: String,
  manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionRuntime {
  #[serde(rename = "type")]
  runtime_type: String,
  entry: String,
  width: Option<u32>,
  height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Action {
  id: String,
  name: String,
  #[serde(default)]
  description: String,
  #[serde(default = "default_version")]
  version: String,
  #[serde(default)]
  author: String,
  #[serde(default)]
  tags: Vec<String>,
  #[serde(default)]
  icon: String,
  #[serde(default = "default_status")]
  status: String,
  #[serde(default)]
  permissions: Vec<String>,
  #[serde(default)]
  components: Vec<String>,
  runtime: ActionRuntime,
  #[serde(default = "default_source")]
  source: ActionSource,
}

#[derive(Debug, Clone, Serialize)]
struct RunResult {
  ok: bool,
  message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallResult {
  action: Action,
  previous_version: Option<String>,
  message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CopyColorSample {
  x: i32,
  y: i32,
  screen_x: i32,
  screen_y: i32,
  hex: String,
  pixels: Vec<u32>,
  clicked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VirtualScreenBounds {
  x: i32,
  y: i32,
  width: u32,
  height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenCapture {
  width: u32,
  height: u32,
  pixels: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeSelection {
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  done: bool,
  cancel: bool,
}

static COPY_COLOR_ACTIVE: AtomicBool = AtomicBool::new(false);
static COPY_COLOR_SESSION: AtomicU64 = AtomicU64::new(0);
static EXTRACT_TEXT_ACTIVE: AtomicBool = AtomicBool::new(false);
static EXTRACT_TEXT_SESSION: AtomicU64 = AtomicU64::new(0);
static WINDOW_ACTION_HTML: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn window_action_html_store() -> &'static Mutex<HashMap<String, String>> {
  WINDOW_ACTION_HTML.get_or_init(|| Mutex::new(HashMap::new()))
}

fn default_version() -> String {
  "0.0.0".to_string()
}

fn default_status() -> String {
  "ready".to_string()
}

fn default_source() -> ActionSource {
  ActionSource {
    kind: String::new(),
    path: String::new(),
    manifest_path: String::new(),
  }
}

fn app_actions_dir(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_data_dir()
    .map(|path| path.join("actions"))
    .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageConvertSelection {
  paths: Vec<String>,
  source_kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageConvertExportResult {
  output_paths: Vec<String>,
  count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoQualityOption {
  id: String,
  label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoPreview {
  title: String,
  platform: String,
  qualities: Vec<VideoQualityOption>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoDownloadResult {
  output_path: String,
}

fn append_debug_log(app: &AppHandle, message: impl AsRef<str>) {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0);

  let log_dir = app
    .path()
    .app_data_dir()
    .unwrap_or_else(|_| std::env::temp_dir().join("luma"));

  let _ = fs::create_dir_all(&log_dir);
  let log_path = log_dir.join("luma-debug.log");

  if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
    let _ = writeln!(file, "[{timestamp}] {}", message.as_ref());
  }
}

fn remove_legacy_bundled_actions(app: &AppHandle) -> Result<(), String> {
  let app_data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
  let migration_marker = app_data_dir.join(".actions-decoupled-from-app");

  if migration_marker.exists() {
    return Ok(());
  }

  let actions_dir = app_data_dir.join("actions");
  for action_id in ["luma.action.copy-color", "luma.action.extract-text"] {
    let action_dir = actions_dir.join(sanitize_action_id(action_id));
    if action_dir.exists() {
      fs::remove_dir_all(&action_dir).map_err(|error| error.to_string())?;
    }
  }

  fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
  fs::write(migration_marker, "Actions were separated from the LUMA base app in 0.1.0.\n")
    .map_err(|error| error.to_string())?;

  Ok(())
}

#[cfg(debug_assertions)]
fn development_actions_dir(app: &AppHandle) -> Result<PathBuf, String> {
  if let Ok(current_dir) = std::env::current_dir() {
    let dev_actions_dir = current_dir.join("actions");
    if dev_actions_dir.exists() {
      return Ok(dev_actions_dir);
    }
  }

  let resource_dir = app
    .path()
    .resource_dir()
    .map_err(|error| error.to_string())?;

  Ok(resource_dir
    .parent()
    .and_then(Path::parent)
    .map(|path| path.join("actions"))
      .unwrap_or_else(|| PathBuf::from("actions")))
}

fn built_in_components_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let mut candidates = Vec::new();

  if let Ok(current_dir) = std::env::current_dir() {
    candidates.push(current_dir.join("components"));
    candidates.push(current_dir.join("_up_").join("components"));
  }

  let resource_dir = app
    .path()
    .resource_dir()
    .map_err(|error| error.to_string())?;

  candidates.push(resource_dir.join("components"));
  candidates.push(resource_dir.join("_up_").join("components"));

  if let Some(parent) = resource_dir.parent() {
    candidates.push(parent.join("components"));
    candidates.push(parent.join("_up_").join("components"));
  }

  if let Ok(exe_path) = std::env::current_exe() {
    if let Some(exe_dir) = exe_path.parent() {
      candidates.push(exe_dir.join("components"));
      candidates.push(exe_dir.join("_up_").join("components"));
    }
  }

  for candidate in &candidates {
    if candidate.exists() {
      append_debug_log(
        app,
        format!("components: using directory {}", candidate.display()),
      );
      return Ok(candidate.clone());
    }
  }

  Err(format!(
    "No se encontro la carpeta de componentes. Rutas probadas: {}",
    candidates
      .iter()
      .map(|path| path.display().to_string())
      .collect::<Vec<_>>()
      .join(" | ")
  ))
}

fn read_action_directory(dir: &Path, source_kind: &str) -> Option<Action> {
  let manifest_path = dir.join("manifest.json");
  let manifest = fs::read_to_string(&manifest_path).ok()?;
  let mut action: Action = serde_json::from_str(&manifest).ok()?;

  action.source = ActionSource {
    kind: source_kind.to_string(),
    path: dir.to_string_lossy().to_string(),
    manifest_path: manifest_path.to_string_lossy().to_string(),
  };

  Some(action)
}

fn read_actions_from_dir(dir: &Path, source_kind: &str) -> Vec<Action> {
  fs::read_dir(dir)
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_dir())
    .filter(|entry| entry.file_name().to_string_lossy() != "template")
    .filter_map(|entry| read_action_directory(&entry.path(), source_kind))
    .collect()
}

fn collect_actions(app: &AppHandle) -> Result<Vec<Action>, String> {
  let installed_dir = app_actions_dir(app)?;
  fs::create_dir_all(&installed_dir).map_err(|error| error.to_string())?;

  let mut actions = read_actions_from_dir(&installed_dir, "installed");

  #[cfg(debug_assertions)]
  {
    let mut seen_ids = actions
      .iter()
      .map(|action| action.id.clone())
      .collect::<std::collections::HashSet<_>>();

    if let Ok(dev_dir) = development_actions_dir(app) {
      for action in read_actions_from_dir(&dev_dir, "development") {
        if seen_ids.insert(action.id.clone()) {
          actions.push(action);
        }
      }
    }
  }

  actions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  Ok(actions)
}

fn center_window_on_cursor(window: &WebviewWindow) -> tauri::Result<()> {
  const LAUNCHER_WIDTH: f64 = 560.0;
  const LAUNCHER_HEIGHT: f64 = 460.0;

  let scale_factor = window.scale_factor().unwrap_or(1.0);
  let physical_width = (LAUNCHER_WIDTH * scale_factor).round() as i32;
  let physical_height = (LAUNCHER_HEIGHT * scale_factor).round() as i32;
  let cursor = cursor_position().unwrap_or((0, 0));
  let monitors = window.available_monitors()?;
  let monitor = monitors
    .iter()
    .find(|monitor| {
      let position = monitor.position();
      let monitor_size = monitor.size();
      cursor.0 >= position.x
        && cursor.0 < position.x + monitor_size.width as i32
        && cursor.1 >= position.y
        && cursor.1 < position.y + monitor_size.height as i32
    })
    .or_else(|| monitors.first());

  if let Some(monitor) = monitor {
    let position = monitor.position();
    let monitor_size = monitor.size();
    let x = position.x + ((monitor_size.width as i32 - physical_width) / 2);
    let y = position.y + ((monitor_size.height as i32 - physical_height) / 2);
    window.set_size(LogicalSize::new(LAUNCHER_WIDTH, LAUNCHER_HEIGHT))?;
    window.set_position(PhysicalPosition::new(x, y))?;
  }

  Ok(())
}

#[cfg(target_os = "windows")]
fn cursor_position() -> Option<(i32, i32)> {
  use windows::Win32::Foundation::POINT;
  use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

  let mut point = POINT { x: 0, y: 0 };

  unsafe {
    if GetCursorPos(&mut point).is_ok() {
      Some((point.x, point.y))
    } else {
      None
    }
  }
}

#[cfg(not(target_os = "windows"))]
fn cursor_position() -> Option<(i32, i32)> {
  None
}

#[cfg(target_os = "windows")]
fn key_is_down(virtual_key: i32) -> bool {
  use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

  unsafe { (GetAsyncKeyState(virtual_key) as u16 & 0x8000) != 0 }
}

fn show_launcher(app: &AppHandle) -> Result<(), String> {
  let window = app
    .get_webview_window("main")
    .ok_or_else(|| "No se encontro la ventana principal.".to_string())?;

  if window.is_visible().map_err(|error| error.to_string())? {
    window.hide().map_err(|error| error.to_string())?;
    return Ok(());
  }

  let actions = collect_actions(app)?;
  center_window_on_cursor(&window).map_err(|error| error.to_string())?;
  window
    .emit("tools-updated", actions)
    .map_err(|error| error.to_string())?;
  window.show().map_err(|error| error.to_string())?;
  window.set_focus().map_err(|error| error.to_string())?;

  Ok(())
}

fn sanitize_action_id(id: &str) -> String {
  id
    .chars()
    .map(|character| {
      if character.is_ascii_alphanumeric() || matches!(character, '.' | '-') {
        character.to_ascii_lowercase()
      } else {
        '-'
      }
    })
    .collect()
}

fn action_window_label(id: &str) -> String {
  let safe_id: String = id
    .chars()
    .map(|character| {
      if character.is_ascii_alphanumeric() || character == '-' {
        character.to_ascii_lowercase()
      } else {
        '-'
      }
    })
    .collect();

  format!("luma-action-{safe_id}")
}

fn validate_lm_path(path: &Path) -> Result<(), String> {
  if path.extension().and_then(|extension| extension.to_str()) != Some("lm") {
    return Err("El archivo debe tener extension .lm.".to_string());
  }

  Ok(())
}

fn read_action_manifest_from_bundle(bundle_path: &Path) -> Result<Action, String> {
  let file = File::open(bundle_path).map_err(|error| error.to_string())?;
  let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
  let mut manifest_file = archive
    .by_name("manifest.json")
    .map_err(|_| "El .lm debe incluir manifest.json en la raiz.".to_string())?;

  let mut manifest = String::new();
  manifest_file
    .read_to_string(&mut manifest)
    .map_err(|error| error.to_string())?;

  serde_json::from_str(&manifest).map_err(|error| error.to_string())
}

fn safe_zip_path(name: &str) -> Result<PathBuf, String> {
  let path = Path::new(name);

  if path.is_absolute()
    || name.contains("..")
    || name.contains(':')
    || name.starts_with('/')
    || name.starts_with('\\')
  {
    return Err(format!("Ruta insegura en el .lm: {name}"));
  }

  Ok(path.to_path_buf())
}

fn extract_action_bundle(bundle_path: &Path, target_dir: &Path) -> Result<(), String> {
  let file = File::open(bundle_path).map_err(|error| error.to_string())?;
  let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;

  if target_dir.exists() {
    fs::remove_dir_all(target_dir).map_err(|error| error.to_string())?;
  }
  fs::create_dir_all(target_dir).map_err(|error| error.to_string())?;

  for index in 0..archive.len() {
    let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
    let relative_path = safe_zip_path(entry.name())?;
    let output_path = target_dir.join(relative_path);

    if entry.is_dir() {
      fs::create_dir_all(&output_path).map_err(|error| error.to_string())?;
      continue;
    }

    if let Some(parent) = output_path.parent() {
      fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let mut output_file = File::create(&output_path).map_err(|error| error.to_string())?;
    std::io::copy(&mut entry, &mut output_file).map_err(|error| error.to_string())?;
  }

  Ok(())
}

fn install_action_bundle(app: &AppHandle, bundle_path: &Path) -> Result<InstallResult, String> {
  validate_lm_path(bundle_path)?;
  let mut action = read_action_manifest_from_bundle(bundle_path)?;

  if !action.id.starts_with("luma.action.") {
    return Err("El id de la Action debe empezar con luma.action.".to_string());
  }

  let action_dir = app_actions_dir(app)?.join(sanitize_action_id(&action.id));
  let previous_action = if action_dir.exists() {
    read_action_directory(&action_dir, "installed")
  } else {
    None
  };

  if let Some(previous) = previous_action.as_ref() {
    if previous.id != action.id {
      return Err("La Action instalada tiene un id distinto al esperado.".to_string());
    }

    if previous.version == action.version {
      return Err(format!(
        "{} {} ya esta instalada. Cambia la version del manifest para actualizarla.",
        action.name, action.version
      ));
    }
  }

  let previous_version = previous_action.map(|previous| previous.version);
  extract_action_bundle(bundle_path, &action_dir)?;

  action.source = ActionSource {
    kind: "installed".to_string(),
    path: action_dir.to_string_lossy().to_string(),
    manifest_path: action_dir.join("manifest.json").to_string_lossy().to_string(),
  };

  let message = if let Some(previous_version) = previous_version.as_ref() {
    format!("{} actualizada: {} -> {}.", action.name, previous_version, action.version)
  } else {
    format!("{} instalada.", action.name)
  };

  Ok(InstallResult {
    action,
    previous_version,
    message,
  })
}

fn action_entry_path(action: &Action) -> Result<PathBuf, String> {
  if !action.runtime.entry.starts_with("action/") {
    return Err("La entrada de la Action debe estar dentro de action/.".to_string());
  }

  let entry_path = Path::new(&action.source.path).join(&action.runtime.entry);

  if !entry_path.exists() {
    return Err(format!("No existe la entrada de la Action: {}", action.runtime.entry));
  }

  Ok(entry_path)
}

fn file_url_for_action(entry_path: &Path) -> Result<Url, String> {
  Url::from_file_path(entry_path)
    .map_err(|_| format!("No se pudo abrir la entrada de la Action: {}", entry_path.display()))
}

fn prepared_window_action_html(entry_path: &Path) -> Result<String, String> {
  let action_dir = entry_path
    .parent()
    .ok_or_else(|| "No se pudo resolver la carpeta de la Action.".to_string())?;
  let mut html = fs::read_to_string(entry_path).map_err(|error| error.to_string())?;

  let stylesheet_path = action_dir.join("styles.css");
  if stylesheet_path.exists() {
    let css = fs::read_to_string(&stylesheet_path).map_err(|error| error.to_string())?;
    html = html
      .replace(
        r#"<link rel="stylesheet" href="./styles.css" />"#,
        &format!("<style>\n{css}\n</style>"),
      )
      .replace(
        r#"<link rel="stylesheet" href="./styles.css">"#,
        &format!("<style>\n{css}\n</style>"),
      );
  }

  let script_path = action_dir.join("action.js");
  if script_path.exists() {
    let js = fs::read_to_string(&script_path).map_err(|error| error.to_string())?;
    html = html
      .replace(
        r#"<script src="./action.js"></script>"#,
        &format!("<script>\n{js}\n</script>"),
      )
      .replace(
        r#"<script src="./action.js" defer></script>"#,
        &format!("<script defer>\n{js}\n</script>"),
      );
  }

  Ok(html)
}

fn window_action_host_url() -> Result<Url, String> {
  let html = r#"<!doctype html>
<html lang="es">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>LUMA Action</title>
    <style>
      :root { color-scheme: dark; --bg:#101012; --ink:#fff; --muted:rgba(255,255,255,.68); --accent:#b8ff2c; --danger:#ff6b6b; --line:rgba(255,255,255,.14); }
      * { box-sizing: border-box; }
      html, body { width: 100%; height: 100%; margin: 0; overflow: hidden; background: var(--bg); color: var(--ink); font-family: "DM Sans", "Inter", "Segoe UI", sans-serif; }
      main { display: grid; min-height: 100%; align-content: center; gap: 12px; padding: 22px; }
      p { margin: 0; }
      .eyebrow { color: var(--accent); font-size: 11px; font-weight: 850; letter-spacing: .08em; text-transform: uppercase; }
      h1 { margin: 0; font-size: 22px; line-height: 1.1; }
      #detail { min-height: 18px; color: var(--muted); font-size: 13px; font-weight: 700; line-height: 1.35; }
      #detail.error { color: var(--danger); }
      button { justify-self: start; min-width: 92px; height: 36px; border: 1px solid var(--line); border-radius: 8px; background: rgba(255,255,255,.08); color: var(--ink); font: inherit; font-weight: 800; }
    </style>
  </head>
  <body>
    <main>
      <p class="eyebrow">LUMA Action</p>
      <h1>Cargando Action...</h1>
      <p id="detail">Preparando la ventana.</p>
      <button id="close" type="button">Cerrar</button>
    </main>
    <script>
      const detail = document.querySelector('#detail');
      const currentWindow = window.__TAURI__?.window?.getCurrentWindow?.();
      const invoke = window.__TAURI__?.core?.invoke;

      function setError(message) {
        detail.textContent = message;
        detail.classList.add('error');
      }

      async function closeHost() {
        if (currentWindow) {
          await currentWindow.close().catch(() => {});
          return;
        }
        window.close();
      }

      document.querySelector('#close').addEventListener('click', () => closeHost());
      window.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
          event.preventDefault();
          closeHost();
        }
      });

      async function bootAction() {
        try {
          if (typeof invoke !== 'function') {
            throw new Error('LUMA no expuso el puente interno para cargar Actions.');
          }

          const actionHtml = await invoke('get_current_window_action_html');
          if (!actionHtml || typeof actionHtml !== 'string') {
            throw new Error('La Action no devolvio contenido HTML.');
          }

          document.open();
          document.write(actionHtml);
          document.close();
        } catch (error) {
          setError(String(error || 'No se pudo cargar la Action.'));
        }
      }

      bootAction();
    </script>
  </body>
</html>"#;

  let html_path = std::env::temp_dir().join(format!(
    "luma-window-action-host-{}.html",
    COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst)
  ));
  fs::write(&html_path, html).map_err(|error| error.to_string())?;
  Url::from_file_path(&html_path).map_err(|_| "No se pudo preparar el host de la Action.".to_string())
}

fn open_action_window(app: &AppHandle, action: &Action) -> Result<(), String> {
  let entry_path = action_entry_path(action)?;
  let label = action_window_label(&action.id);

  if let Some(window) = app.get_webview_window(&label) {
    let _ = window.close();
    thread::sleep(Duration::from_millis(80));
  }

  let html = prepared_window_action_html(&entry_path)?;
  window_action_html_store()
    .lock()
    .map_err(|error| error.to_string())?
    .insert(label.clone(), html);
  let width = action.runtime.width.unwrap_or(520) as f64;
  let height = action.runtime.height.unwrap_or(360) as f64;

  let host_url = window_action_host_url()?;
  let window = WebviewWindowBuilder::new(app, label.clone(), WebviewUrl::External(host_url))
    .title(action.name.clone())
    .inner_size(width, height)
    .resizable(false)
    .decorations(true)
    .always_on_top(true)
    .center()
    .build()
    .map_err(|error| error.to_string())?;

  if let Some(icon) = app.default_window_icon().cloned() {
    let _ = window.set_icon(icon);
  }

  let _ = window.set_ignore_cursor_events(false);

  Ok(())
}

fn open_action_overlay(app: &AppHandle, action: &Action) -> Result<(), String> {
  let entry_path = action_entry_path(action)?;
  let label = action_window_label(&action.id);
  let url = file_url_for_action(&entry_path)?;
  let (screen_x, screen_y, screen_width, screen_height) = virtual_screen_bounds();

  if let Some(window) = app.get_webview_window(&label) {
    let _ = window.set_ignore_cursor_events(false);
    window
      .set_position(PhysicalPosition::new(screen_x, screen_y))
      .map_err(|error| error.to_string())?;
    window
      .set_size(PhysicalSize::new(screen_width, screen_height))
      .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window
      .emit("luma-overlay-start", serde_json::json!({ "x": screen_x, "y": screen_y }))
      .map_err(|error| error.to_string())?;
    start_extract_text_native_selection(app, &action.id, &label, screen_x, screen_y);
    return Ok(());
  }

  let window = WebviewWindowBuilder::new(app, label.clone(), WebviewUrl::External(url))
    .title(action.name.clone())
    .position(screen_x as f64, screen_y as f64)
    .inner_size(screen_width as f64, screen_height as f64)
    .decorations(false)
    .resizable(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .build()
    .map_err(|error| error.to_string())?;

  if let Some(icon) = app.default_window_icon().cloned() {
    let _ = window.set_icon(icon);
  }

  let _ = window.set_ignore_cursor_events(false);
  window.set_focus().map_err(|error| error.to_string())?;
  window
    .emit("luma-overlay-start", serde_json::json!({ "x": screen_x, "y": screen_y }))
    .map_err(|error| error.to_string())?;
  start_extract_text_native_selection(app, &action.id, &label, screen_x, screen_y);

  Ok(())
}

fn start_extract_text_native_selection(
  app: &AppHandle,
  action_id: &str,
  label: &str,
  screen_x: i32,
  screen_y: i32,
) {
  if !matches!(
    action_id,
    "luma.action.extract-text" | "luma.action.translate-image"
  ) {
    return;
  }

  EXTRACT_TEXT_ACTIVE.store(true, Ordering::SeqCst);
  let session = EXTRACT_TEXT_SESSION.fetch_add(1, Ordering::SeqCst) + 1;
  run_extract_text_selection_sampler(
    app.clone(),
    label.to_string(),
    action_id.to_string(),
    session,
    screen_x,
    screen_y,
  );
}

fn preload_ocr_component(app: &AppHandle) -> Result<(), String> {
  let label = "luma-component-ocr";
  if app.get_webview_window(label).is_some() {
    return Ok(());
  }

  let entry_path = built_in_components_dir(app)?.join("ocr").join("index.html");
  if !entry_path.exists() {
    return Err(format!("No existe el componente OCR: {}", entry_path.display()));
  }

  let url = file_url_for_action(&entry_path)?;
  let window = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
    .title("LUMA OCR Component")
    .inner_size(320.0, 240.0)
    .decorations(false)
    .resizable(false)
    .visible(false)
    .skip_taskbar(true)
    .build()
    .map_err(|error| error.to_string())?;

  if let Some(icon) = app.default_window_icon().cloned() {
    let _ = window.set_icon(icon);
  }

  let _ = window.emit("luma-component-ocr-warmup", serde_json::json!({}));

  Ok(())
}

fn preload_declared_components(app: &AppHandle) -> Result<(), String> {
  let actions = collect_actions(app)?;

  if actions
    .iter()
    .any(|action| action.components.iter().any(|component| component == "luma.component.ocr"))
  {
    preload_ocr_component(app)?;
  }

  Ok(())
}

fn preload_extract_text_overlay(app: &AppHandle) -> Result<(), String> {
  let (screen_x, screen_y, screen_width, screen_height) = virtual_screen_bounds();

  for action in collect_actions(app)?.into_iter().filter(|action| {
    matches!(
      action.id.as_str(),
      "luma.action.extract-text" | "luma.action.translate-image"
    )
  }) {
    let label = action_window_label(&action.id);
    if app.get_webview_window(&label).is_some() {
      continue;
    }

    let entry_path = action_entry_path(&action)?;
    let url = file_url_for_action(&entry_path)?;

    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
      .title(action.name)
      .position(screen_x as f64, screen_y as f64)
      .inner_size(screen_width as f64, screen_height as f64)
      .decorations(false)
      .resizable(false)
      .transparent(true)
      .always_on_top(true)
      .skip_taskbar(true)
      .shadow(false)
      .visible(false)
      .build()
      .map_err(|error| error.to_string())?;

    if let Some(icon) = app.default_window_icon().cloned() {
      let _ = window.set_icon(icon);
    }
  }

  Ok(())
}

#[tauri::command]
fn get_current_window_action_html(window: WebviewWindow) -> Result<String, String> {
  let label = window.label().to_string();
  window_action_html_store()
    .lock()
    .map_err(|error| error.to_string())?
    .get(&label)
    .cloned()
    .ok_or_else(|| format!("No hay HTML preparado para la ventana {label}."))
}

#[cfg(target_os = "windows")]
fn rgb_to_hex(color: u32) -> String {
  let color = color & 0x00FF_FFFF;
  let red = (color >> 16) & 0xFF;
  let green = (color >> 8) & 0xFF;
  let blue = color & 0xFF;
  format!("#{red:02X}{green:02X}{blue:02X}")
}

#[cfg(target_os = "windows")]
fn virtual_screen_bounds() -> (i32, i32, u32, u32) {
  use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN,
  };

  unsafe {
    (
      GetSystemMetrics(SM_XVIRTUALSCREEN),
      GetSystemMetrics(SM_YVIRTUALSCREEN),
      GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32,
      GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32,
    )
  }
}

#[tauri::command]
fn get_virtual_screen_bounds() -> VirtualScreenBounds {
  let (x, y, width, height) = virtual_screen_bounds();

  VirtualScreenBounds {
    x,
    y,
    width,
    height,
  }
}

#[cfg(target_os = "windows")]
fn capture_screen_region_pixels(x: i32, y: i32, width: u32, height: u32) -> Result<ScreenCapture, String> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
  };

  if width < 1 || height < 1 {
    return Err("Seleccion demasiado pequena.".to_string());
  }

  let mut bgra = vec![0u8; (width as usize) * (height as usize) * 4];

  unsafe {
    let hwnd = HWND(std::ptr::null_mut());
    let screen_hdc = GetDC(hwnd);
    let memory_hdc = CreateCompatibleDC(screen_hdc);
    let bitmap = CreateCompatibleBitmap(screen_hdc, width as i32, height as i32);
    let old_object = SelectObject(memory_hdc, bitmap);

    let result = (|| -> Result<(), String> {
      BitBlt(memory_hdc, 0, 0, width as i32, height as i32, screen_hdc, x, y, SRCCOPY)
        .map_err(|error| error.to_string())?;

      let mut info = BITMAPINFO::default();
      info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        biHeight: -(height as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..BITMAPINFOHEADER::default()
      };

      let read_lines = GetDIBits(
        memory_hdc,
        bitmap,
        0,
        height,
        Some(bgra.as_mut_ptr().cast::<core::ffi::c_void>()),
        &mut info,
        DIB_RGB_COLORS,
      );

      if read_lines != height as i32 {
        return Err("No se pudo capturar el fragmento seleccionado.".to_string());
      }

      Ok(())
    })();

    let _ = SelectObject(memory_hdc, old_object);
    let _ = DeleteObject(bitmap);
    let _ = DeleteDC(memory_hdc);
    let _ = ReleaseDC(hwnd, screen_hdc);
    result?;
  }

  for pixel in bgra.chunks_exact_mut(4) {
    pixel.swap(0, 2);
    pixel[3] = 255;
  }

  Ok(ScreenCapture {
    width,
    height,
    pixels: bgra,
  })
}

#[cfg(not(target_os = "windows"))]
fn capture_screen_region_pixels(_x: i32, _y: i32, _width: u32, _height: u32) -> Result<ScreenCapture, String> {
  Err("La captura de pantalla todavia solo esta implementada en Windows.".to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn capture_screen_region(x: i32, y: i32, width: u32, height: u32) -> Result<ScreenCapture, String> {
  capture_screen_region_pixels(x, y, width, height)
}

#[cfg(target_os = "windows")]
fn recognize_image_with_windows_ocr(image_path: &Path) -> Result<String, String> {
  let script = r#"
param([string]$ImagePath)

Add-Type -AssemblyName System.Runtime.WindowsRuntime
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [Console]::OutputEncoding

function Await-WinRt($Operation, [Type]$ResultType) {
  $method = [System.WindowsRuntimeSystemExtensions].GetMethods() |
    Where-Object {
      $_.Name -eq 'AsTask' -and
      $_.IsGenericMethodDefinition -and
      $_.GetParameters().Count -eq 1
    } |
    Select-Object -First 1

  $task = $method.MakeGenericMethod($ResultType).Invoke($null, @($Operation))
  $task.Wait()
  return $task.Result
}

[Windows.Storage.StorageFile, Windows.Storage, ContentType=WindowsRuntime] | Out-Null
[Windows.Storage.FileAccessMode, Windows.Storage, ContentType=WindowsRuntime] | Out-Null
[Windows.Graphics.Imaging.BitmapDecoder, Windows.Graphics.Imaging, ContentType=WindowsRuntime] | Out-Null
[Windows.Graphics.Imaging.SoftwareBitmap, Windows.Graphics.Imaging, ContentType=WindowsRuntime] | Out-Null
[Windows.Media.Ocr.OcrEngine, Windows.Foundation, ContentType=WindowsRuntime] | Out-Null
[Windows.Globalization.Language, Windows.Globalization, ContentType=WindowsRuntime] | Out-Null

$file = Await-WinRt ([Windows.Storage.StorageFile]::GetFileFromPathAsync($ImagePath)) ([Windows.Storage.StorageFile])
$stream = Await-WinRt ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
$decoder = Await-WinRt ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
$bitmap = Await-WinRt ($decoder.GetSoftwareBitmapAsync()) ([Windows.Graphics.Imaging.SoftwareBitmap])

function Format-OcrResult($Result) {
  $lines = @($Result.Lines)

  if ($lines.Count -eq 0) {
    return ($Result.Text -as [string]).Trim()
  }

  $minLeft = [double]::PositiveInfinity
  foreach ($line in $lines) {
    foreach ($word in @($line.Words)) {
      if ($word.BoundingRect.X -lt $minLeft) {
        $minLeft = $word.BoundingRect.X
      }
    }
  }

  if ([double]::IsPositiveInfinity($minLeft)) {
    return ($Result.Text -as [string]).Trim()
  }

  $formattedLines = New-Object System.Collections.Generic.List[string]
  foreach ($line in $lines) {
    $words = @($line.Words)
    $text = ($line.Text -as [string]).TrimEnd()

    if ([string]::IsNullOrWhiteSpace($text)) {
      $formattedLines.Add('')
      continue
    }

    $lineLeft = $minLeft
    if ($words.Count -gt 0) {
      $lineLeft = ($words | ForEach-Object { $_.BoundingRect.X } | Measure-Object -Minimum).Minimum
    }

    $indentWidth = [Math]::Max(0, [Math]::Round(($lineLeft - $minLeft) / 16))
    $indentWidth = [Math]::Min(12, [int]$indentWidth)
    $formattedLines.Add((' ' * $indentWidth) + $text)
  }

  return ($formattedLines -join "`r`n").Trim()
}

function Score-OcrText([string]$Text) {
  if ([string]::IsNullOrWhiteSpace($Text)) {
    return -100000
  }

  $codeSymbols = ([regex]::Matches($Text, '[\{\}\[\]\(\);=<>]').Count)
  $braces = ([regex]::Matches($Text, '[\{\}]').Count)
  $lineBreaks = ([regex]::Matches($Text, '\r?\n').Count)
  $replacementMarks = ([regex]::Matches($Text, [char]0xFFFD).Count)

  return $Text.Length + ($codeSymbols * 80) + ($braces * 220) + ($lineBreaks * 12) - ($replacementMarks * 120)
}

function Repair-CodeSymbols([string]$Text) {
  if ([string]::IsNullOrWhiteSpace($Text)) {
    return $Text
  }

  $codeSignals = ([regex]::Matches($Text, '(?i)\b(function|const|let|var|return|if|else|for|while|switch|class|interface|type|import|export|try|catch|finally|async|await)\b|=>|;|=|:|\(|\)|\[|\]').Count)
  if ($codeSignals -lt 2) {
    return $Text
  }

  $text = $Text -replace ([string][char]0xFFFD), ''
  $text = $text -replace 'ï¿½', ''
  $lines = $text -split '\r?\n'
  $fixedLines = New-Object System.Collections.Generic.List[string]

  foreach ($rawLine in $lines) {
    $line = $rawLine.TrimEnd()
    $trimmed = $line.Trim()

    if ($trimmed.Length -eq 0) {
      $fixedLines.Add($line)
      continue
    }

    $line = [regex]::Replace($line, '\)\s*[;:]\s*$', '}')
    $line = [regex]::Replace($line, '\(\s*[;:]\s*$', '{')
    $line = [regex]::Replace($line, '^\s*\)\s*$', '}')
    $line = [regex]::Replace($line, '^\s*\(\s*$', '{')

    $trimmed = $line.Trim()
    $opensBlock =
      $trimmed -match '(?i)^(if|else\s+if|for|foreach|while|switch|catch|with)\b.*\)\s*$' -or
      $trimmed -match '(?i)^(else|try|finally|do)\s*$' -or
      $trimmed -match '(?i)^(function|class|interface|enum)\b.*[A-Za-z0-9_\)\]]\s*$' -or
      $trimmed -match '=>\s*$' -or
      $trimmed -match '=\s*$' -or
      $trimmed -match ':\s*$'

    if ($opensBlock -and $trimmed -notmatch '\{\s*$') {
      $line = $line + ' {'
    }

    $fixedLines.Add($line)
  }

  $text = ($fixedLines -join "`r`n")
  if (([regex]::Matches($text, '[\{\}]').Count) -eq 0) {
    $text = [regex]::Replace(
      $text,
      '(?m)^(\s*(?:const|let|var)\s+[A-Za-z_$][\w$]*\s*=\s*)([^;\r\n]*:[^;\r\n]*)(;?)\s*$',
      '$1{ $2 }$3'
    )
    $text = [regex]::Replace(
      $text,
      '(?m)^(\s*return\s+)([^;\r\n]*:[^;\r\n]*)(;?)\s*$',
      '$1{ $2 }$3'
    )
    $text = [regex]::Replace(
      $text,
      '(?m)^(\s*[A-Za-z_$][\w$]*\s*:\s*)([^,\r\n]+,\s*)$',
      '$1{ $2 }'
    )
  }

  $openBraces = ([regex]::Matches($text, '\{').Count)
  $closeBraces = ([regex]::Matches($text, '\}').Count)
  if ($openBraces -gt $closeBraces) {
    $missing = $openBraces - $closeBraces
    return $text.TrimEnd() + ("`r`n" + ('}' * $missing))
  }

  return $text
}

$engines = New-Object System.Collections.Generic.List[object]

foreach ($tag in @('es-ES', 'es-MX', 'en-US')) {
  try {
    $language = [Windows.Globalization.Language]::new($tag)
    $engine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromLanguage($language)
    if ($null -ne $engine) {
      $engines.Add($engine)
    }
  } catch {
  }
}

$profileEngine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
if ($null -ne $profileEngine) {
  $engines.Add($profileEngine)
}

if ($engines.Count -eq 0) {
  throw 'Windows OCR is not available for the current user languages.'
}

$bestText = ''
$bestScore = -100000
foreach ($engine in $engines) {
  $result = Await-WinRt ($engine.RecognizeAsync($bitmap)) ([Windows.Media.Ocr.OcrResult])
  $text = Format-OcrResult $result
  $score = Score-OcrText $text

  if ($score -gt $bestScore) {
    $bestScore = $score
    $bestText = Repair-CodeSymbols $text
  }
}

[Console]::Out.Write($bestText.Trim())
"#;
  let script_path = std::env::temp_dir().join(format!(
    "luma-ocr-{}-{}.ps1",
    std::process::id(),
    COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst)
  ));

  fs::write(&script_path, script).map_err(|error| error.to_string())?;

  let mut child = Command::new("powershell")
    .args([
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      &script_path.to_string_lossy(),
      &image_path.to_string_lossy(),
    ])
    .creation_flags(0x08000000)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| error.to_string())?;

  let started_at = Instant::now();
  loop {
    if child.try_wait().map_err(|error| error.to_string())?.is_some() {
      break;
    }

    if started_at.elapsed() > Duration::from_secs(8) {
      let _ = child.kill();
      let _ = child.wait();
      let _ = fs::remove_file(&script_path);
      return Err("Windows OCR tardo demasiado.".to_string());
    }

    thread::sleep(Duration::from_millis(25));
  }

  let output = child
    .wait_with_output()
    .map_err(|error| error.to_string())?;

  let _ = fs::remove_file(&script_path);

  if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
    return Err(if error.is_empty() {
      "Windows OCR failed.".to_string()
    } else {
      error
    });
  }

  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(target_os = "windows")]
fn ocr_screen_region_blocking(x: i32, y: i32, width: u32, height: u32) -> Result<String, String> {
  let image_path = save_screen_region_for_ocr(x, y, width, height)?;
  let result = recognize_image_with_windows_ocr(&image_path);
  let _ = fs::remove_file(&image_path);
  result
}

#[cfg(target_os = "windows")]
fn save_screen_region_for_ocr(x: i32, y: i32, width: u32, height: u32) -> Result<PathBuf, String> {
  let capture = capture_screen_region_pixels(x, y, width, height)?;
  let image_path = std::env::temp_dir().join(format!(
    "luma-ocr-{}-{}.png",
    std::process::id(),
    COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst)
  ));

  let image = image::RgbaImage::from_raw(capture.width, capture.height, capture.pixels)
    .ok_or_else(|| "No se pudo preparar la imagen para OCR.".to_string())?;
  let scale = if capture.width.max(capture.height) <= 1800 { 2 } else { 1 };
  let image = if scale > 1 {
    image::imageops::resize(
      &image,
      capture.width.saturating_mul(scale),
      capture.height.saturating_mul(scale),
      image::imageops::FilterType::CatmullRom,
    )
  } else {
    image
  };

  image
    .save_with_format(&image_path, image::ImageFormat::Png)
    .map_err(|error| error.to_string())?;

  Ok(image_path)
}

#[cfg(target_os = "windows")]
#[tauri::command(rename_all = "camelCase")]
async fn ocr_screen_region(x: i32, y: i32, width: u32, height: u32) -> Result<String, String> {
  tauri::async_runtime::spawn_blocking(move || ocr_screen_region_blocking(x, y, width, height))
    .await
    .map_err(|error| error.to_string())?
}

fn normalize_translation_target(target_language: &str) -> Result<&'static str, String> {
  match target_language.trim().to_ascii_lowercase().as_str() {
    "es" | "spanish" | "espanol" | "español" => Ok("es"),
    "en" | "english" | "ingles" | "inglés" => Ok("en"),
    "pt" | "portuguese" | "portugues" | "portugués" => Ok("pt"),
    "fr" | "french" | "frances" | "francés" => Ok("fr"),
    "it" | "italian" | "italiano" => Ok("it"),
    "de" | "german" | "aleman" | "alemán" => Ok("de"),
    "ja" | "japanese" | "japones" | "japonés" => Ok("ja"),
    _ => Err("Idioma de destino no soportado.".to_string()),
  }
}

fn translate_text_blocking(text: &str, target_language: &str) -> Result<String, String> {
  let text = text.trim();
  if text.is_empty() {
    return Ok(String::new());
  }
  let target_language = normalize_translation_target(target_language)?;

  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(8))
    .user_agent("LUMA/0.1.0")
    .build()
    .map_err(|error| error.to_string())?;

  let response = client
    .get("https://translate.googleapis.com/translate_a/single")
    .query(&[
      ("client", "gtx"),
      ("sl", "auto"),
      ("tl", target_language),
      ("dt", "t"),
      ("q", text),
    ])
    .send()
    .map_err(|error| format!("No se pudo conectar al traductor: {error}"))?;

  if !response.status().is_success() {
    return Err(format!("El traductor respondio con estado {}.", response.status()));
  }

  let payload: serde_json::Value = response.json().map_err(|error| error.to_string())?;
  let Some(segments) = payload.get(0).and_then(|value| value.as_array()) else {
    return Err("El traductor devolvio una respuesta inesperada.".to_string());
  };

  let mut translated = String::new();
  for segment in segments {
    if let Some(part) = segment.get(0).and_then(|value| value.as_str()) {
      translated.push_str(part);
    }
  }

  Ok(translated.trim().to_string())
}

fn translate_text_to_spanish_blocking(text: &str) -> Result<String, String> {
  translate_text_blocking(text, "es")
}

#[tauri::command(rename_all = "camelCase")]
async fn translate_text_to_spanish(text: String) -> Result<String, String> {
  tauri::async_runtime::spawn_blocking(move || translate_text_to_spanish_blocking(&text))
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(rename_all = "camelCase")]
async fn translate_text(app: AppHandle, text: String, target_language: String) -> Result<String, String> {
  append_debug_log(
    &app,
    format!(
      "translate_text: requested target={} chars={}",
      target_language,
      text.chars().count()
    ),
  );
  let log_app = app.clone();
  let log_target = target_language.clone();
  tauri::async_runtime::spawn_blocking(move || translate_text_blocking(&text, &target_language))
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result)
    .map(|translated| {
      append_debug_log(
        &log_app,
        format!(
          "translate_text: completed target={} chars={}",
          log_target,
          translated.chars().count()
        ),
      );
      translated
    })
    .map_err(|error| {
      append_debug_log(&log_app, format!("translate_text: failed: {error}"));
      error
    })
}

#[tauri::command(rename_all = "camelCase")]
fn luma_debug_log(app: AppHandle, window: WebviewWindow, message: String) -> Result<(), String> {
  append_debug_log(
    &app,
    format!("frontend window={} message={}", window.label(), message),
  );
  Ok(())
}

fn html_escape(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

fn open_translate_image_result_window(app: AppHandle, title: String, body: String, is_error: bool) {
  let app_for_window = app.clone();
  let _ = app.run_on_main_thread(move || {
    let label = "luma-translate-image-result";
    if let Some(window) = app_for_window.get_webview_window(label) {
      let title_json = serde_json::to_string(&title).unwrap_or_else(|_| "\"Translate Image\"".to_string());
      let body_json = serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".to_string());
      let is_error_js = if is_error { "true" } else { "false" };
      let _ = window.eval(&format!(
        "window.__setTranslateImageResult && window.__setTranslateImageResult({title_json}, {body_json}, {is_error_js});"
      ));
      let _ = window.show();
      let _ = window.set_focus();
      return;
    }

    let escaped_title = html_escape(&title);
    let escaped_body = html_escape(&body);
    let status_class = if is_error { "error" } else { "" };
    let copy_disabled = if is_error || body.trim().is_empty() { "disabled" } else { "" };

    let html = format!(
      r#"<!doctype html>
<html lang="es">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Translate Image</title>
    <style>
      :root {{ color-scheme: dark; --panel:#121214; --ink:#fff; --muted:rgba(255,255,255,.68); --line:rgba(255,255,255,.12); --accent:#b8ff2c; --danger:#ff6b6b; }}
      * {{ box-sizing: border-box; }}
      html, body {{ width: 100%; height: 100%; margin: 0; overflow: hidden; background: transparent; font-family: "DM Sans", "Inter", "Segoe UI", sans-serif; }}
      main {{ display: grid; grid-template-rows: auto 1fr auto; gap: 14px; width: 100vw; height: 100vh; padding: 18px; border: 1px solid var(--line); border-radius: 10px; background: var(--panel); color: var(--ink); }}
      header, footer {{ display: flex; align-items: center; justify-content: space-between; gap: 14px; }}
      p {{ margin: 0; }}
      .eyebrow {{ margin-bottom: 5px; color: var(--accent); font-size: 11px; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }}
      h1 {{ margin: 0; font-size: 24px; font-weight: 800; line-height: 1.1; }}
      #close {{ width: 34px; height: 34px; border: 1px solid var(--line); border-radius: 8px; background: rgba(255,255,255,.06); color: var(--ink); font: inherit; font-size: 22px; line-height: 1; }}
      pre {{ width: 100%; min-height: 0; margin: 0; padding: 14px; overflow: auto; border: 1px solid var(--line); border-radius: 8px; background: rgba(0,0,0,.22); color: var(--ink); font-family: "Cascadia Mono", Consolas, monospace; font-size: 13px; line-height: 1.45; white-space: pre-wrap; user-select: text; }}
      #status {{ min-width: 0; color: var(--muted); font-size: 12px; font-weight: 700; line-height: 1.3; }}
      #status.error {{ color: var(--danger); }}
      #copy {{ min-width: 104px; height: 38px; border: 0; border-radius: 8px; background: var(--ink); color: #101010; font: inherit; font-size: 13px; font-weight: 800; }}
      #copy:disabled {{ cursor: not-allowed; opacity: .42; }}
    </style>
  </head>
  <body>
    <main>
      <header>
        <div><p class="eyebrow">Translate Image</p><h1>{escaped_title}</h1></div>
        <button id="close" type="button" aria-label="Cerrar">&times;</button>
      </header>
      <pre id="text">{escaped_body}</pre>
      <footer>
        <p id="status" class="{status_class}">{escaped_title}</p>
        <button id="copy" type="button" {copy_disabled}>Copiar</button>
      </footer>
    </main>
    <script>
      let currentText = document.querySelector('#text').textContent;
      window.__setTranslateImageResult = (title, body, isError) => {{
        currentText = body;
        document.querySelector('h1').textContent = title;
        document.querySelector('#text').textContent = body;
        const status = document.querySelector('#status');
        status.textContent = title;
        status.classList.toggle('error', isError);
        document.querySelector('#copy').disabled = isError || body.trim().length === 0;
      }};
      const closeWindow = () => window.close();
      document.querySelector('#close').addEventListener('click', closeWindow);
      window.addEventListener('keydown', (event) => {{
        if (event.key === 'Escape') {{
          event.preventDefault();
          closeWindow();
        }}
      }});
      document.querySelector('#copy').addEventListener('click', async () => {{
        try {{
          await navigator.clipboard.writeText(currentText);
          document.querySelector('#status').textContent = 'Copiado.';
        }} catch (_error) {{
          const range = document.createRange();
          range.selectNodeContents(document.querySelector('#text'));
          const selection = window.getSelection();
          selection.removeAllRanges();
          selection.addRange(range);
          document.execCommand('copy');
          selection.removeAllRanges();
          document.querySelector('#status').textContent = 'Copiado.';
        }}
      }});
    </script>
  </body>
</html>"#
    );

    let html_path = std::env::temp_dir().join(format!(
      "{label}-{}.html",
      COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst)
    ));
    if fs::write(&html_path, html).is_err() {
      return;
    }

    let Ok(url) = Url::from_file_path(&html_path) else {
      return;
    };

    let Ok(window) = WebviewWindowBuilder::new(&app_for_window, label, WebviewUrl::External(url))
      .title("Translate Image")
      .inner_size(520.0, 390.0)
      .resizable(false)
      .decorations(true)
      .always_on_top(true)
      .center()
      .build()
    else {
      return;
    };

    if let Some(icon) = app_for_window.default_window_icon().cloned() {
      let _ = window.set_icon(icon);
    }
    let _ = window.set_focus();
  });
}

fn open_translate_this_window(app: &AppHandle) -> Result<(), String> {
  append_debug_log(app, "translate-this: open requested");
  let window = app
    .get_webview_window("luma-translate-this")
    .ok_or_else(|| "No se encontro la ventana predeclarada de Translate This.".to_string())?;

  append_debug_log(app, "translate-this: predeclared window found");
  window.set_size(PhysicalSize::new(520, 420)).map_err(|error| {
    append_debug_log(app, format!("translate-this: set size failed: {error}"));
    error.to_string()
  })?;
  window.center().map_err(|error| {
    append_debug_log(app, format!("translate-this: center failed: {error}"));
    error.to_string()
  })?;
  window.show().map_err(|error| {
    append_debug_log(app, format!("translate-this: show failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "translate-this: shown");
  window.set_focus().map_err(|error| {
    append_debug_log(app, format!("translate-this: focus failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "translate-this: focus set");
  Ok(())
}

fn open_count_characters_window(app: &AppHandle) -> Result<(), String> {
  append_debug_log(app, "contar-caracteres: open requested");
  let window = app
    .get_webview_window("luma-contar-caracteres")
    .ok_or_else(|| "No se encontro la ventana predeclarada de ContarCaracteres.".to_string())?;

  append_debug_log(app, "contar-caracteres: predeclared window found");
  window.set_size(PhysicalSize::new(520, 430)).map_err(|error| {
    append_debug_log(app, format!("contar-caracteres: set size failed: {error}"));
    error.to_string()
  })?;
  window.center().map_err(|error| {
    append_debug_log(app, format!("contar-caracteres: center failed: {error}"));
    error.to_string()
  })?;
  window.show().map_err(|error| {
    append_debug_log(app, format!("contar-caracteres: show failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "contar-caracteres: shown");
  window.set_focus().map_err(|error| {
    append_debug_log(app, format!("contar-caracteres: focus failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "contar-caracteres: focus set");
  Ok(())
}

fn open_merge_pdf_window(app: &AppHandle) -> Result<(), String> {
  append_debug_log(app, "merge-pdf: open requested");
  let window = app
    .get_webview_window("luma-merge-pdf")
    .ok_or_else(|| "No se encontro la ventana predeclarada de Merge PDF.".to_string())?;

  append_debug_log(app, "merge-pdf: predeclared window found");
  window.set_size(PhysicalSize::new(540, 460)).map_err(|error| {
    append_debug_log(app, format!("merge-pdf: set size failed: {error}"));
    error.to_string()
  })?;
  window.center().map_err(|error| {
    append_debug_log(app, format!("merge-pdf: center failed: {error}"));
    error.to_string()
  })?;
  window.show().map_err(|error| {
    append_debug_log(app, format!("merge-pdf: show failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "merge-pdf: shown");
  window.set_focus().map_err(|error| {
    append_debug_log(app, format!("merge-pdf: focus failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "merge-pdf: focus set");
  Ok(())
}

fn open_image_convert_window(app: &AppHandle) -> Result<(), String> {
  append_debug_log(app, "image-convert: open requested");
  let window = app
    .get_webview_window("luma-image-convert")
    .ok_or_else(|| "No se encontro la ventana predeclarada de Image Convert.".to_string())?;

  append_debug_log(app, "image-convert: predeclared window found");
  window.set_size(PhysicalSize::new(560, 520)).map_err(|error| {
    append_debug_log(app, format!("image-convert: set size failed: {error}"));
    error.to_string()
  })?;
  window.center().map_err(|error| {
    append_debug_log(app, format!("image-convert: center failed: {error}"));
    error.to_string()
  })?;
  window.show().map_err(|error| {
    append_debug_log(app, format!("image-convert: show failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "image-convert: shown");
  window.set_focus().map_err(|error| {
    append_debug_log(app, format!("image-convert: focus failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "image-convert: focus set");
  Ok(())
}

fn open_video_downloader_window(app: &AppHandle) -> Result<(), String> {
  append_debug_log(app, "video-downloader: open requested");
  let window = app
    .get_webview_window("luma-video-downloader")
    .ok_or_else(|| "No se encontro la ventana predeclarada de Video Downloader.".to_string())?;

  append_debug_log(app, "video-downloader: predeclared window found");
  window.set_size(PhysicalSize::new(560, 500)).map_err(|error| {
    append_debug_log(app, format!("video-downloader: set size failed: {error}"));
    error.to_string()
  })?;
  window.center().map_err(|error| {
    append_debug_log(app, format!("video-downloader: center failed: {error}"));
    error.to_string()
  })?;
  window.show().map_err(|error| {
    append_debug_log(app, format!("video-downloader: show failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "video-downloader: shown");
  window.set_focus().map_err(|error| {
    append_debug_log(app, format!("video-downloader: focus failed: {error}"));
    error.to_string()
  })?;
  append_debug_log(app, "video-downloader: focus set");
  Ok(())
}

#[cfg(target_os = "windows")]
fn translate_image_region_to_window(
  app: AppHandle,
  label: String,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
) {
  thread::spawn(move || {
    if let Some(window) = app.get_webview_window(&label) {
      let _ = window.hide();
    }

    thread::sleep(Duration::from_millis(80));

    let image_path = match save_screen_region_for_ocr(x, y, width, height) {
      Ok(image_path) => image_path,
      Err(error) => {
        open_translate_image_result_window(
          app,
          "No se pudo traducir".to_string(),
          format!("No se pudo capturar la seleccion: {error}"),
          true,
        );
        return;
      }
    };

    open_translate_image_result_window(
      app.clone(),
      "Leyendo texto".to_string(),
      "OCR en proceso...".to_string(),
      false,
    );

    let text = match recognize_image_with_windows_ocr(&image_path) {
      Ok(text) => text,
      Err(error) => {
        let _ = fs::remove_file(&image_path);
        open_translate_image_result_window(
          app,
          "No se pudo traducir".to_string(),
          format!("No se pudo leer el texto: {error}"),
          true,
        );
        return;
      }
    };
    let _ = fs::remove_file(&image_path);

    if text.trim().is_empty() {
      open_translate_image_result_window(
        app,
        "Traduccion lista".to_string(),
        "No se detecto texto para traducir.".to_string(),
        false,
      );
      return;
    }

    match translate_text_to_spanish_blocking(&text) {
      Ok(translated) => open_translate_image_result_window(
        app,
        "Traduccion lista".to_string(),
        translated,
        false,
      ),
      Err(error) => open_translate_image_result_window(
        app,
        "No se pudo traducir".to_string(),
        format!("No se pudo traducir: {error}"),
        true,
      ),
    }
  });
}

#[cfg(target_os = "windows")]
#[tauri::command(rename_all = "camelCase")]
fn translate_image_from_screen_region(
  app: AppHandle,
  window: WebviewWindow,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
) -> Result<(), String> {
  translate_image_region_to_window(app, window.label().to_string(), x, y, width, height);
  Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command(rename_all = "camelCase")]
fn translate_image_from_screen_region(
  _app: AppHandle,
  _window: WebviewWindow,
  _x: i32,
  _y: i32,
  _width: u32,
  _height: u32,
) -> Result<(), String> {
  Err("Translate Image todavia solo esta implementado en Windows.".to_string())
}

#[tauri::command(rename_all = "camelCase")]
fn show_current_window_panel(window: WebviewWindow, width: u32, height: u32) -> Result<(), String> {
  window
    .set_size(PhysicalSize::new(width, height))
    .map_err(|error| error.to_string())?;
  window.center().map_err(|error| error.to_string())?;
  window.show().map_err(|error| error.to_string())?;
  window.set_focus().map_err(|error| error.to_string())?;
  Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command(rename_all = "camelCase")]
fn extract_text_from_screen_region(x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
  thread::spawn(move || {
    let Ok(text) = ocr_screen_region_blocking(x, y, width, height) else {
      return;
    };

    let text = text.trim();
    if text.is_empty() {
      return;
    }

    if let Ok(mut clipboard) = arboard::Clipboard::new() {
      let _ = clipboard.set_text(text.to_string());
    }
  });

  Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command(rename_all = "camelCase")]
async fn ocr_screen_region(_x: i32, _y: i32, _width: u32, _height: u32) -> Result<String, String> {
  Err("OCR nativo todavia solo esta implementado en Windows.".to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command(rename_all = "camelCase")]
fn extract_text_from_screen_region(_x: i32, _y: i32, _width: u32, _height: u32) -> Result<(), String> {
  Err("OCR nativo todavia solo esta implementado en Windows.".to_string())
}

#[cfg(target_os = "windows")]
fn run_extract_text_selection_sampler(
  app: AppHandle,
  label: String,
  action_id: String,
  session: u64,
  _origin_x: i32,
  _origin_y: i32,
) {
  thread::spawn(move || {
    let Some(window) = app.get_webview_window(&label) else {
      return;
    };

    let mut dragging = false;
    let mut ready_for_press = !key_is_down(0x01);
    let mut start_x = 0;
    let mut start_y = 0;
    let selection_window = create_extract_text_native_selection_window();
    let mut shown_selection_window = false;
    let mut last_frame: Option<(i32, i32, u32, u32)> = None;

    loop {
      if !EXTRACT_TEXT_ACTIVE.load(Ordering::SeqCst)
        || EXTRACT_TEXT_SESSION.load(Ordering::SeqCst) != session
      {
        hide_extract_text_native_selection_window(selection_window);
        break;
      }

      if key_is_down(0x1B) {
        EXTRACT_TEXT_ACTIVE.store(false, Ordering::SeqCst);
        hide_extract_text_native_selection_window(selection_window);
        let _ = window.emit("luma-native-selection", NativeSelection {
          x: 0,
          y: 0,
          width: 0,
          height: 0,
          done: false,
          cancel: true,
        });
        let _ = window.hide();
        break;
      }

      let is_pressed = key_is_down(0x01);
      if !is_pressed {
        ready_for_press = true;
      }

      let Some((cursor_x, cursor_y)) = cursor_position() else {
        thread::sleep(Duration::from_millis(8));
        continue;
      };

      if ready_for_press && is_pressed && !dragging {
        dragging = true;
        start_x = cursor_x;
        start_y = cursor_y;
      }

      if dragging {
        let left = start_x.min(cursor_x);
        let top = start_y.min(cursor_y);
        let width = start_x.abs_diff(cursor_x);
        let height = start_y.abs_diff(cursor_y);
        let next_frame = if width > 2 && height > 2 {
          Some((left, top, width, height))
        } else {
          None
        };

        if last_frame != next_frame {
          if let Some(frame) = next_frame {
            shown_selection_window = show_extract_text_native_selection_window(selection_window, frame);
          } else if shown_selection_window {
            hide_extract_text_native_selection_window(selection_window);
            shown_selection_window = false;
          }
          last_frame = next_frame;
        }

        if !is_pressed {
          EXTRACT_TEXT_ACTIVE.store(false, Ordering::SeqCst);
          hide_extract_text_native_selection_window(selection_window);

          if width >= 8 && height >= 8 {
            if action_id == "luma.action.translate-image" {
              let _ = window.hide();
              translate_image_region_to_window(app.clone(), label.clone(), left, top, width, height);
            } else {
              let _ = window.hide();
              let _ = extract_text_from_screen_region(left, top, width, height);
            }
          } else {
            let _ = window.hide();
          }

          break;
        }
      }

      thread::sleep(Duration::from_millis(3));
    }
  });
}

#[cfg(not(target_os = "windows"))]
fn run_extract_text_selection_sampler(
  _app: AppHandle,
  _label: String,
  _action_id: String,
  _session: u64,
  _origin_x: i32,
  _origin_y: i32,
) {
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn extract_text_selection_window_proc(
  hwnd: windows::Win32::Foundation::HWND,
  msg: u32,
  wparam: windows::Win32::Foundation::WPARAM,
  lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
  use windows::Win32::{
    Foundation::{COLORREF, LRESULT, RECT},
    Graphics::Gdi::{
      BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, PAINTSTRUCT,
    },
    UI::WindowsAndMessaging::{DefWindowProcW, GetClientRect, WM_ERASEBKGND, WM_PAINT},
  };

  match msg {
    WM_ERASEBKGND => LRESULT(1),
    WM_PAINT => {
      let mut paint = PAINTSTRUCT::default();
      let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
      if !hdc.0.is_null() {
        let mut rect = RECT::default();
        if unsafe { GetClientRect(hwnd, &mut rect) }.is_ok() {
          let black = unsafe { CreateSolidBrush(COLORREF(0x000000)) };
          let cyan = unsafe { CreateSolidBrush(COLORREF(0x00FFD926)) };
          let _ = unsafe { FillRect(hdc, &rect, black) };

          let width = rect.right - rect.left;
          let height = rect.bottom - rect.top;
          let thickness = 3;

          if width > thickness * 2 && height > thickness * 2 {
            let top = RECT {
              left: 0,
              top: 0,
              right: width,
              bottom: thickness,
            };
            let bottom = RECT {
              left: 0,
              top: height - thickness,
              right: width,
              bottom: height,
            };
            let left = RECT {
              left: 0,
              top: 0,
              right: thickness,
              bottom: height,
            };
            let right = RECT {
              left: width - thickness,
              top: 0,
              right: width,
              bottom: height,
            };

            let _ = unsafe { FillRect(hdc, &top, cyan) };
            let _ = unsafe { FillRect(hdc, &bottom, cyan) };
            let _ = unsafe { FillRect(hdc, &left, cyan) };
            let _ = unsafe { FillRect(hdc, &right, cyan) };
          }

          let _ = unsafe { DeleteObject(black) };
          let _ = unsafe { DeleteObject(cyan) };
        }
      }
      let _ = unsafe { EndPaint(hwnd, &paint) };
      LRESULT(0)
    }
    _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
  }
}

#[cfg(target_os = "windows")]
fn create_extract_text_native_selection_window() -> Option<windows::Win32::Foundation::HWND> {
  use windows::{
    core::w,
    Win32::{
      Foundation::{COLORREF, HINSTANCE, HWND},
      Graphics::Gdi::{InvalidateRect, UpdateWindow},
      UI::WindowsAndMessaging::{
        CreateWindowExW, RegisterClassW, SetLayeredWindowAttributes, ShowWindow, HMENU,
        LWA_COLORKEY, SW_HIDE, WINDOW_EX_STYLE, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
      },
    },
  };

  unsafe {
    let class_name = w!("LumaExtractTextNativeSelection");
    let window_class = WNDCLASSW {
      lpfnWndProc: Some(extract_text_selection_window_proc),
      hInstance: HINSTANCE(std::ptr::null_mut()),
      lpszClassName: class_name,
      ..Default::default()
    };
    let _ = RegisterClassW(&window_class);

    let hwnd = CreateWindowExW(
      WINDOW_EX_STYLE(
        WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0 | WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0,
      ),
      class_name,
      class_name,
      WS_POPUP,
      0,
      0,
      10,
      10,
      HWND(std::ptr::null_mut()),
      HMENU(std::ptr::null_mut()),
      HINSTANCE(std::ptr::null_mut()),
      None,
    )
    .ok()?;

    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0x000000), 255, LWA_COLORKEY);
    let _ = InvalidateRect(hwnd, None, true);
    let _ = UpdateWindow(hwnd);
    let _ = ShowWindow(hwnd, SW_HIDE);

    Some(hwnd)
  }
}

#[cfg(target_os = "windows")]
fn show_extract_text_native_selection_window(
  hwnd: Option<windows::Win32::Foundation::HWND>,
  (left, top, width, height): (i32, i32, u32, u32),
) -> bool {
  use windows::Win32::{
    Graphics::Gdi::{InvalidateRect, UpdateWindow},
    UI::WindowsAndMessaging::{
      SetWindowPos, HWND_TOPMOST, SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    },
  };

  let Some(hwnd) = hwnd else {
    return false;
  };

  if width < 3 || height < 3 {
    return false;
  }

  unsafe {
    let flags = SET_WINDOW_POS_FLAGS(SWP_NOACTIVATE.0 | SWP_SHOWWINDOW.0);
    if SetWindowPos(
      hwnd,
      HWND_TOPMOST,
      left,
      top,
      width as i32,
      height as i32,
      flags,
    )
    .is_err()
    {
      return false;
    }

    let _ = InvalidateRect(hwnd, None, true);
    let _ = UpdateWindow(hwnd);
  }

  true
}

#[cfg(target_os = "windows")]
fn hide_extract_text_native_selection_window(hwnd: Option<windows::Win32::Foundation::HWND>) {
  use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};

  if let Some(hwnd) = hwnd {
    unsafe {
      let _ = ShowWindow(hwnd, SW_HIDE);
    }
  }
}

#[cfg(target_os = "windows")]
#[cfg(target_os = "windows")]
fn copy_color_sample_from_capture(
  screen_hdc: windows::Win32::Graphics::Gdi::HDC,
  memory_hdc: windows::Win32::Graphics::Gdi::HDC,
  bitmap: windows::Win32::Graphics::Gdi::HBITMAP,
) -> Result<CopyColorSample, String> {
  use windows::Win32::Graphics::Gdi::{
    BitBlt, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
  };

  let (screen_x, screen_y) = cursor_position().ok_or_else(|| "No se pudo ubicar el cursor.".to_string())?;
  let mut pixels = vec![0u32; 81];

  unsafe {
    BitBlt(memory_hdc, 0, 0, 9, 9, screen_hdc, screen_x - 4, screen_y - 4, SRCCOPY)
      .map_err(|error| error.to_string())?;

    let mut info = BITMAPINFO::default();
    info.bmiHeader = BITMAPINFOHEADER {
      biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: 9,
      biHeight: -9,
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..BITMAPINFOHEADER::default()
    };

    let read_lines = GetDIBits(
      memory_hdc,
      bitmap,
      0,
      9,
      Some(pixels.as_mut_ptr().cast::<core::ffi::c_void>()),
      &mut info,
      DIB_RGB_COLORS,
    );

    if read_lines != 9 {
      return Err("No se pudo leer la lupa de color.".to_string());
    }

    pixels.iter_mut().for_each(|pixel| *pixel &= 0x00FF_FFFF);
    let center = pixels[40];

    Ok(CopyColorSample {
      x: 0,
      y: 0,
      screen_x,
      screen_y,
      hex: rgb_to_hex(center),
      pixels,
      clicked: false,
    })
  }
}

#[cfg(target_os = "windows")]
fn copy_color_sample_at_cursor() -> Result<CopyColorSample, String> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject,
  };

  unsafe {
    let hwnd = HWND(std::ptr::null_mut());
    let screen_hdc = GetDC(hwnd);
    let memory_hdc = CreateCompatibleDC(screen_hdc);
    let bitmap = CreateCompatibleBitmap(screen_hdc, 9, 9);
    let old_object = SelectObject(memory_hdc, bitmap);
    let sample = copy_color_sample_from_capture(screen_hdc, memory_hdc, bitmap);
    let _ = SelectObject(memory_hdc, old_object);
    let _ = DeleteObject(bitmap);
    let _ = DeleteDC(memory_hdc);
    let _ = ReleaseDC(hwnd, screen_hdc);
    sample
  }
}

fn activate_copy_color(app: &AppHandle) -> Result<(), String> {
  if cursor_position().is_none() {
    return Err("No se pudo ubicar el cursor.".to_string());
  }
  COPY_COLOR_ACTIVE.store(true, Ordering::SeqCst);
  let session = COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst) + 1;

  let (screen_x, screen_y, screen_width, screen_height) = virtual_screen_bounds();

  let window = app
    .get_webview_window("copy-color-overlay")
    .ok_or_else(|| "No se encontro la UI de Copiar color.".to_string())?;

  let _ = window.set_ignore_cursor_events(false);
  window
    .set_position(PhysicalPosition::new(screen_x, screen_y))
    .map_err(|error| error.to_string())?;
  window
    .set_size(PhysicalSize::new(screen_width, screen_height))
    .map_err(|error| error.to_string())?;
  window.show().map_err(|error| error.to_string())?;
  window.set_focus().map_err(|error| error.to_string())?;
  window
    .emit("copy-color-start", serde_json::json!({ "x": screen_x, "y": screen_y }))
    .map_err(|error| error.to_string())?;
  run_copy_color_sampler(app.clone(), session);

  Ok(())
}

#[cfg(not(target_os = "windows"))]
fn activate_copy_color(_app: &AppHandle) -> Result<(), String> {
  Err("Copiar color nativo solo esta implementado en Windows por ahora.".to_string())
}

#[cfg(target_os = "windows")]
fn run_copy_color_sampler(app: AppHandle, session: u64) {
  thread::spawn(move || {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
      CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, ReleaseDC,
      SelectObject,
    };

    let hwnd = HWND(std::ptr::null_mut());
    let screen_hdc = unsafe { GetDC(hwnd) };
    let memory_hdc = unsafe { CreateCompatibleDC(screen_hdc) };
    let bitmap = unsafe { CreateCompatibleBitmap(screen_hdc, 9, 9) };
    let old_object = unsafe { SelectObject(memory_hdc, bitmap) };

    loop {
      if !COPY_COLOR_ACTIVE.load(Ordering::SeqCst)
        || COPY_COLOR_SESSION.load(Ordering::SeqCst) != session
      {
        break;
      }

      let Some(window) = app.get_webview_window("copy-color-overlay") else {
        break;
      };

      if let Ok(sample) = copy_color_sample_from_capture(screen_hdc, memory_hdc, bitmap) {
        let _ = window.emit("copy-color-sample", sample);
      }

      thread::sleep(Duration::from_millis(8));
    }

    let _ = unsafe { SelectObject(memory_hdc, old_object) };
    let _ = unsafe { DeleteObject(bitmap) };
    let _ = unsafe { DeleteDC(memory_hdc) };
    let _ = unsafe { ReleaseDC(hwnd, screen_hdc) };
  });
}

#[tauri::command]
fn sample_copy_color(_app: AppHandle) -> Result<CopyColorSample, String> {
  copy_color_sample_at_cursor()
}

#[tauri::command]
fn finish_copy_color(app: AppHandle, hex: String) -> Result<(), String> {
  COPY_COLOR_ACTIVE.store(false, Ordering::SeqCst);
  let _ = COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst);

  if let Some(window) = app.get_webview_window("copy-color-overlay") {
    window.hide().map_err(|error| error.to_string())?;
  }

  thread::spawn(move || {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
      let _ = clipboard.set_text(hex);
    }
  });

  Ok(())
}

#[tauri::command]
fn cancel_copy_color(app: AppHandle) -> Result<(), String> {
  COPY_COLOR_ACTIVE.store(false, Ordering::SeqCst);
  let _ = COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst);
  if let Some(window) = app.get_webview_window("copy-color-overlay") {
    window.hide().map_err(|error| error.to_string())?;
  }

  Ok(())
}

#[tauri::command]
fn write_clipboard_text(text: String) -> Result<(), String> {
  let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
  clipboard.set_text(text).map_err(|error| error.to_string())
}

fn pdf_object_type(object: &Object) -> &[u8] {
  object
    .as_dict()
    .ok()
    .and_then(|dictionary| dictionary.get(b"Type").ok())
    .and_then(|object| object.as_name().ok())
    .unwrap_or(b"")
}

fn merge_pdf_documents(input_paths: &[PathBuf], output_path: &Path) -> Result<(), String> {
  if input_paths.len() < 2 {
    return Err("Selecciona al menos dos PDF.".to_string());
  }

  let mut max_id = 1;
  let mut source_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
  let mut source_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();

  for path in input_paths {
    if !path.is_file() {
      return Err(format!("No se encontro el archivo: {}", path.display()));
    }

    let extension = path
      .extension()
      .and_then(|extension| extension.to_str())
      .unwrap_or_default();
    if !extension.eq_ignore_ascii_case("pdf") {
      return Err(format!("El archivo no es PDF: {}", path.display()));
    }

    let mut document = Document::load(path)
      .map_err(|error| format!("No se pudo leer {}: {error}", path.display()))?;
    document.renumber_objects_with(max_id);
    max_id = document.max_id + 1;

    for (_page_number, page_id) in document.get_pages() {
      let page = document
        .get_object(page_id)
        .map_err(|error| format!("No se pudo leer una pagina de {}: {error}", path.display()))?;
      source_pages.insert(page_id, page.to_owned());
    }

    source_objects.extend(document.objects);
  }

  if source_pages.is_empty() {
    return Err("Los PDF seleccionados no tienen paginas para unir.".to_string());
  }

  let highest_existing_id = source_objects
    .keys()
    .map(|(id, _generation)| *id)
    .max()
    .unwrap_or(max_id);
  let pages_id = (highest_existing_id + 1, 0);
  let catalog_id = (highest_existing_id + 2, 0);
  let mut merged = Document::with_version("1.5");

  for (object_id, object) in source_objects {
    match pdf_object_type(&object) {
      b"Catalog" | b"Pages" | b"Page" => {}
      _ => {
        merged.objects.insert(object_id, object);
      }
    }
  }

  let mut page_ids = Vec::with_capacity(source_pages.len());
  for (page_id, page_object) in source_pages {
    let mut page_dictionary = page_object
      .as_dict()
      .map_err(|error| format!("No se pudo preparar una pagina del PDF: {error}"))?
      .clone();
    page_dictionary.set("Parent", Object::Reference(pages_id));
    merged
      .objects
      .insert(page_id, Object::Dictionary(page_dictionary));
    page_ids.push(page_id);
  }

  let mut pages_dictionary = Dictionary::new();
  pages_dictionary.set("Type", "Pages");
  pages_dictionary.set(
    "Kids",
    Object::Array(
      page_ids
        .iter()
        .map(|page_id| Object::Reference(*page_id))
        .collect(),
    ),
  );
  pages_dictionary.set("Count", page_ids.len() as i64);
  merged
    .objects
    .insert(pages_id, Object::Dictionary(pages_dictionary));

  let mut catalog_dictionary = Dictionary::new();
  catalog_dictionary.set("Type", "Catalog");
  catalog_dictionary.set("Pages", Object::Reference(pages_id));
  merged
    .objects
    .insert(catalog_id, Object::Dictionary(catalog_dictionary));
  merged.trailer.set("Root", Object::Reference(catalog_id));
  merged.max_id = catalog_id.0;
  merged.renumber_objects();
  merged.compress();

  merged
    .save(output_path)
    .map(|_| ())
    .map_err(|error| format!("No se pudo guardar el PDF unido: {error}"))
}

#[tauri::command]
fn merge_pdf_pick_files(app: AppHandle) -> Result<Vec<String>, String> {
  append_debug_log(&app, "merge-pdf: pick files requested");
  let merge_window = app.get_webview_window("luma-merge-pdf");
  if let Some(window) = merge_window.as_ref() {
    let _ = window.set_always_on_top(false);
  }

  let selected_paths = rfd::FileDialog::new()
    .set_title("Seleccionar PDF")
    .add_filter("PDF", &["pdf"])
    .pick_files();

  if let Some(window) = merge_window.as_ref() {
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
  }

  let Some(paths) = selected_paths else {
    append_debug_log(&app, "merge-pdf: pick files canceled");
    return Ok(Vec::new());
  };

  let selected = paths
    .into_iter()
    .map(|path| path.to_string_lossy().to_string())
    .collect::<Vec<_>>();
  append_debug_log(
    &app,
    format!("merge-pdf: picked files count={}", selected.len()),
  );
  Ok(selected)
}

#[tauri::command]
fn merge_pdf_export(app: AppHandle, files: Vec<String>) -> Result<String, String> {
  append_debug_log(
    &app,
    format!("merge-pdf: export requested files={}", files.len()),
  );

  let input_paths = files.into_iter().map(PathBuf::from).collect::<Vec<_>>();
  if input_paths.len() < 2 {
    return Err("Selecciona al menos dos PDF.".to_string());
  }

  let merge_window = app.get_webview_window("luma-merge-pdf");
  if let Some(window) = merge_window.as_ref() {
    let _ = window.set_always_on_top(false);
  }

  let selected_output_path = rfd::FileDialog::new()
    .set_title("Guardar PDF unido")
    .add_filter("PDF", &["pdf"])
    .set_file_name("LUMA_Merge_PDF.pdf")
    .save_file();

  if let Some(window) = merge_window.as_ref() {
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
  }

  let Some(mut output_path) = selected_output_path else {
    append_debug_log(&app, "merge-pdf: save dialog canceled");
    return Err("Guardado cancelado.".to_string());
  };

  if output_path.extension().is_none() {
    output_path.set_extension("pdf");
  }

  merge_pdf_documents(&input_paths, &output_path)?;
  append_debug_log(
    &app,
    format!("merge-pdf: export completed path={}", output_path.display()),
  );
  Ok(output_path.to_string_lossy().to_string())
}

fn is_supported_image_path(path: &Path) -> bool {
  path
    .extension()
    .and_then(|extension| extension.to_str())
    .map(|extension| {
      matches!(
        extension.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "ico"
      )
    })
    .unwrap_or(false)
}

fn image_format_from_key(format: &str) -> Result<(image::ImageFormat, &'static str), String> {
  match format.trim().to_ascii_lowercase().as_str() {
    "png" => Ok((image::ImageFormat::Png, "png")),
    "jpg" | "jpeg" => Ok((image::ImageFormat::Jpeg, "jpg")),
    "webp" => Ok((image::ImageFormat::WebP, "webp")),
    "ico" => Ok((image::ImageFormat::Ico, "ico")),
    _ => Err("Formato de salida no soportado.".to_string()),
  }
}

fn sanitized_prefix(prefix: Option<String>) -> String {
  prefix
    .unwrap_or_default()
    .chars()
    .map(|character| match character {
      '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
      _ => character,
    })
    .collect::<String>()
    .trim()
    .to_string()
}

fn unique_output_path(directory: &Path, stem: &str, extension: &str) -> PathBuf {
  let mut candidate = directory.join(format!("{stem}.{extension}"));
  let mut counter = 2;

  while candidate.exists() {
    candidate = directory.join(format!("{stem}-{counter}.{extension}"));
    counter += 1;
  }

  candidate
}

fn convert_image_file(input_path: &Path, output_path: &Path, format: image::ImageFormat) -> Result<(), String> {
  let image = image::open(input_path)
    .map_err(|error| format!("No se pudo leer {}: {error}", input_path.display()))?;

  match format {
    image::ImageFormat::Jpeg => image::DynamicImage::ImageRgb8(image.to_rgb8())
      .save_with_format(output_path, format)
      .map_err(|error| format!("No se pudo guardar {}: {error}", output_path.display())),
    image::ImageFormat::Ico => {
      let icon = if image.width() > 256 || image.height() > 256 {
        image.resize(256, 256, image::imageops::FilterType::Lanczos3)
      } else {
        image
      };

      image::DynamicImage::ImageRgba8(icon.to_rgba8())
        .save_with_format(output_path, format)
        .map_err(|error| format!("No se pudo guardar {}: {error}", output_path.display()))
    }
    _ => image
      .save_with_format(output_path, format)
      .map_err(|error| format!("No se pudo guardar {}: {error}", output_path.display())),
  }
}

#[tauri::command(rename_all = "camelCase")]
fn image_convert_pick_input(app: AppHandle, folder_mode: bool) -> Result<ImageConvertSelection, String> {
  append_debug_log(
    &app,
    format!("image-convert: pick input requested folder_mode={folder_mode}"),
  );
  let convert_window = app.get_webview_window("luma-image-convert");
  if let Some(window) = convert_window.as_ref() {
    let _ = window.set_always_on_top(false);
  }

  let paths = if folder_mode {
    let selected_folder = rfd::FileDialog::new()
      .set_title("Seleccionar carpeta con imagenes")
      .pick_folder();

    if let Some(folder) = selected_folder {
      fs::read_dir(&folder)
        .map_err(|error| format!("No se pudo leer la carpeta: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_supported_image_path(path))
        .collect::<Vec<_>>()
    } else {
      Vec::new()
    }
  } else {
    rfd::FileDialog::new()
      .set_title("Seleccionar imagen")
      .add_filter("Imagen", &["png", "jpg", "jpeg", "webp", "ico"])
      .pick_file()
      .map(|path| vec![path])
      .unwrap_or_default()
  };

  if let Some(window) = convert_window.as_ref() {
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
  }

  let selected = paths
    .into_iter()
    .map(|path| path.to_string_lossy().to_string())
    .collect::<Vec<_>>();

  append_debug_log(
    &app,
    format!("image-convert: picked input count={}", selected.len()),
  );

  Ok(ImageConvertSelection {
    paths: selected,
    source_kind: if folder_mode { "folder" } else { "file" }.to_string(),
  })
}

#[tauri::command(rename_all = "camelCase")]
fn image_convert_export(
  app: AppHandle,
  files: Vec<String>,
  output_format: String,
  prefix: Option<String>,
) -> Result<ImageConvertExportResult, String> {
  append_debug_log(
    &app,
    format!(
      "image-convert: export requested files={} format={}",
      files.len(),
      output_format
    ),
  );

  if files.is_empty() {
    return Err("Selecciona una imagen o una carpeta con imagenes.".to_string());
  }

  let (format, extension) = image_format_from_key(&output_format)?;
  let input_paths = files.into_iter().map(PathBuf::from).collect::<Vec<_>>();
  let prefix = sanitized_prefix(prefix);

  for path in &input_paths {
    if !path.is_file() || !is_supported_image_path(path) {
      return Err(format!("El archivo no es una imagen soportada: {}", path.display()));
    }
  }

  let convert_window = app.get_webview_window("luma-image-convert");
  if let Some(window) = convert_window.as_ref() {
    let _ = window.set_always_on_top(false);
  }

  let output_paths = if input_paths.len() == 1 {
    let input_path = &input_paths[0];
    let stem = input_path
      .file_stem()
      .and_then(|stem| stem.to_str())
      .unwrap_or("imagen");
    let default_name = format!("{prefix}{stem}.{extension}");
    let selected_output = rfd::FileDialog::new()
      .set_title("Guardar imagen convertida")
      .add_filter("Imagen", &[extension])
      .set_file_name(&default_name)
      .save_file();

    selected_output.map(|mut output_path| {
      output_path.set_extension(extension);
      vec![output_path]
    })
  } else {
    rfd::FileDialog::new()
      .set_title("Seleccionar carpeta de destino")
      .pick_folder()
      .map(|folder| {
        input_paths
          .iter()
          .map(|input_path| {
            let stem = input_path
              .file_stem()
              .and_then(|stem| stem.to_str())
              .unwrap_or("imagen");
            unique_output_path(&folder, &format!("{prefix}{stem}"), extension)
          })
          .collect::<Vec<_>>()
      })
  };

  if let Some(window) = convert_window.as_ref() {
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
  }

  let Some(output_paths) = output_paths else {
    append_debug_log(&app, "image-convert: export destination canceled");
    return Err("Guardado cancelado.".to_string());
  };

  for (input_path, output_path) in input_paths.iter().zip(output_paths.iter()) {
    convert_image_file(input_path, output_path, format)?;
  }

  let saved = output_paths
    .iter()
    .map(|path| path.to_string_lossy().to_string())
    .collect::<Vec<_>>();
  append_debug_log(
    &app,
    format!("image-convert: export completed count={}", saved.len()),
  );

  Ok(ImageConvertExportResult {
    count: saved.len(),
    output_paths: saved,
  })
}

fn video_downloader_binary_path(app: &AppHandle) -> Result<PathBuf, String> {
  let binary_path = built_in_components_dir(app)?
    .join("video-downloader")
    .join("yt-dlp.exe");

  if binary_path.is_file() {
    Ok(binary_path)
  } else {
    Err(format!(
      "No se encontro el componente Video Downloader: {}",
      binary_path.display()
    ))
  }
}

fn run_video_downloader_process(
  binary_path: &Path,
  args: &[String],
  timeout: Duration,
) -> Result<(String, String), String> {
  let mut command = Command::new(binary_path);
  command
    .args(args)
    .creation_flags(0x08000000)
    .stdout(Stdio::piped())
    .stderr(Stdio::null());

  let mut child = command.spawn().map_err(|error| error.to_string())?;
  let started_at = Instant::now();

  loop {
    if child.try_wait().map_err(|error| error.to_string())?.is_some() {
      break;
    }

    if started_at.elapsed() > timeout {
      let _ = child.kill();
      let _ = child.wait();
      return Err("Video Downloader tardo demasiado.".to_string());
    }

    thread::sleep(Duration::from_millis(100));
  }

  let output = child.wait_with_output().map_err(|error| error.to_string())?;
  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

  if output.status.success() {
    Ok((stdout, String::new()))
  } else {
    Err("No se pudo procesar el enlace.".to_string())
  }
}

fn run_video_downloader_json_to_file(
  binary_path: &Path,
  args: &[String],
  output_path: &Path,
  timeout: Duration,
) -> Result<String, String> {
  let output_file = File::create(output_path).map_err(|error| error.to_string())?;
  let mut command = Command::new(binary_path);
  command
    .args(args)
    .creation_flags(0x08000000)
    .stdout(Stdio::from(output_file))
    .stderr(Stdio::piped());

  let mut child = command.spawn().map_err(|error| error.to_string())?;
  let started_at = Instant::now();

  loop {
    if child.try_wait().map_err(|error| error.to_string())?.is_some() {
      break;
    }

    if started_at.elapsed() > timeout {
      let _ = child.kill();
      let _ = child.wait();
      let _ = fs::remove_file(output_path);
      return Err("Video Downloader tardo demasiado detectando el video.".to_string());
    }

    thread::sleep(Duration::from_millis(100));
  }

  let output = child.wait_with_output().map_err(|error| error.to_string())?;
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

  if !output.status.success() {
    let _ = fs::remove_file(output_path);
    if stderr.is_empty() {
      return Err("No se pudo procesar el enlace.".to_string());
    }
    return Err(stderr.lines().last().unwrap_or(&stderr).to_string());
  }

  fs::read_to_string(output_path).map_err(|error| error.to_string())
}

fn platform_label_from_info(info: &serde_json::Value, url: &str) -> String {
  let raw = info
    .get("extractor_key")
    .or_else(|| info.get("extractor"))
    .and_then(|value| value.as_str())
    .unwrap_or_default()
    .to_ascii_lowercase();
  let url = url.to_ascii_lowercase();

  if raw.contains("youtube") || url.contains("youtu.be") || url.contains("youtube.com") {
    "youtube"
  } else if raw.contains("instagram") || url.contains("instagram.com") {
    "instagram"
  } else if raw.contains("twitter") || raw == "x" || url.contains("twitter.com") || url.contains("x.com") {
    "twitter"
  } else if raw.contains("tiktok") || url.contains("tiktok.com") {
    "tiktok"
  } else if raw.contains("vimeo") || url.contains("vimeo.com") {
    "vimeo"
  } else if raw.contains("facebook") || url.contains("facebook.com") || url.contains("fb.watch") {
    "facebook"
  } else {
    "video"
  }
  .to_string()
}

fn video_quality_options(info: &serde_json::Value) -> Vec<VideoQualityOption> {
  let mut heights = BTreeSet::new();

  if let Some(formats) = info.get("formats").and_then(|value| value.as_array()) {
    for format in formats {
      let Some(height) = format.get("height").and_then(|value| value.as_u64()) else {
        continue;
      };

      if height >= 144 {
        heights.insert(height);
      }
    }
  }

  let mut options = vec![VideoQualityOption {
    id: "best".to_string(),
    label: "Mejor calidad".to_string(),
  }];

  for height in heights.into_iter().rev().take(8) {
    options.push(VideoQualityOption {
      id: format!("{height}p"),
      label: format!("Hasta {height}p"),
    });
  }

  options
}

fn safe_download_name(title: &str) -> String {
  let sanitized = title
    .chars()
    .map(|character| match character {
      '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
      character if character.is_control() => '_',
      _ => character,
    })
    .collect::<String>()
    .trim()
    .trim_matches('.')
    .to_string();

  if sanitized.is_empty() {
    "video".to_string()
  } else {
    sanitized.chars().take(90).collect()
  }
}

fn format_selector_for_quality(quality: &str) -> String {
  if quality == "best" {
    "best[ext=mp4][vcodec!=none][acodec!=none]/best[vcodec!=none][acodec!=none]/best".to_string()
  } else if let Some(height) = quality.strip_suffix('p').and_then(|value| value.parse::<u32>().ok()) {
    format!(
      "best[height<={height}][ext=mp4][vcodec!=none][acodec!=none]/best[height<={height}][vcodec!=none][acodec!=none]/best[height<={height}]/best"
    )
  } else {
    "best[ext=mp4][vcodec!=none][acodec!=none]/best[vcodec!=none][acodec!=none]/best".to_string()
  }
}

#[tauri::command(rename_all = "camelCase")]
async fn video_downloader_preview(app: AppHandle, url: String) -> Result<VideoPreview, String> {
  tauri::async_runtime::spawn_blocking(move || video_downloader_preview_blocking(app, url))
    .await
    .map_err(|error| error.to_string())?
}

fn video_downloader_preview_blocking(app: AppHandle, url: String) -> Result<VideoPreview, String> {
  let url = url.trim().to_string();
  if url.is_empty() {
    return Err("Pega un enlace para detectar el video.".to_string());
  }

  append_debug_log(&app, format!("video-downloader: preview requested url={url}"));
  let binary_path = video_downloader_binary_path(&app)?;
  let args = vec![
    "--dump-single-json".to_string(),
    "--no-playlist".to_string(),
    "--no-warnings".to_string(),
    url.clone(),
  ];
  let preview_path = std::env::temp_dir().join(format!(
    "luma-video-preview-{}-{}.json",
    std::process::id(),
    COPY_COLOR_SESSION.fetch_add(1, Ordering::SeqCst)
  ));
  let json = run_video_downloader_json_to_file(
    &binary_path,
    &args,
    &preview_path,
    Duration::from_secs(90),
  )?;
  let _ = fs::remove_file(&preview_path);
  let info: serde_json::Value = serde_json::from_str(&json)
    .map_err(|error| format!("No se pudo leer la informacion del video: {error}"))?;
  let title = info
    .get("title")
    .and_then(|value| value.as_str())
    .unwrap_or("Video detectado")
    .to_string();
  let platform = platform_label_from_info(&info, &url);
  let qualities = video_quality_options(&info);

  append_debug_log(
    &app,
    format!(
      "video-downloader: preview completed platform={} qualities={}",
      platform,
      qualities.len()
    ),
  );

  Ok(VideoPreview {
    title,
    platform,
    qualities,
  })
}

#[tauri::command(rename_all = "camelCase")]
async fn video_downloader_download(
  app: AppHandle,
  url: String,
  quality: String,
  title: Option<String>,
) -> Result<VideoDownloadResult, String> {
  tauri::async_runtime::spawn_blocking(move || {
    video_downloader_download_blocking(app, url, quality, title)
  })
  .await
  .map_err(|error| error.to_string())?
}

fn video_downloader_download_blocking(
  app: AppHandle,
  url: String,
  quality: String,
  title: Option<String>,
) -> Result<VideoDownloadResult, String> {
  let url = url.trim().to_string();
  if url.is_empty() {
    return Err("Pega un enlace para descargar.".to_string());
  }

  append_debug_log(
    &app,
    format!("video-downloader: download requested quality={quality}"),
  );
  let binary_path = video_downloader_binary_path(&app)?;
  let video_window = app.get_webview_window("luma-video-downloader");
  if let Some(window) = video_window.as_ref() {
    let _ = window.set_always_on_top(false);
  }

  let default_name = format!("{}.mp4", safe_download_name(title.as_deref().unwrap_or("video")));
  let selected_output = rfd::FileDialog::new()
    .set_title("Guardar video")
    .add_filter("Video", &["mp4", "webm", "mkv", "mov"])
    .set_file_name(&default_name)
    .save_file();

  if let Some(window) = video_window.as_ref() {
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
  }

  let Some(selected_output) = selected_output else {
    append_debug_log(&app, "video-downloader: save dialog canceled");
    return Err("Guardado cancelado.".to_string());
  };

  let output_dir = selected_output
    .parent()
    .map(Path::to_path_buf)
    .ok_or_else(|| "No se pudo resolver la carpeta de destino.".to_string())?;
  let output_stem = selected_output
    .file_stem()
    .and_then(|stem| stem.to_str())
    .map(safe_download_name)
    .unwrap_or_else(|| "video".to_string());
  let output_template = output_dir.join(format!("{output_stem}.%(ext)s"));
  let format_selector = format_selector_for_quality(&quality);
  let args = vec![
    "--no-playlist".to_string(),
    "--windows-filenames".to_string(),
    "--no-warnings".to_string(),
    "-f".to_string(),
    format_selector,
    "-o".to_string(),
    output_template.to_string_lossy().to_string(),
    "--print".to_string(),
    "after_move:filepath".to_string(),
    url,
  ];
  let (stdout, _stderr) = run_video_downloader_process(&binary_path, &args, Duration::from_secs(900))?;
  let output_path = stdout
    .lines()
    .rev()
    .find(|line| !line.trim().is_empty())
    .map(|line| line.trim().to_string())
    .unwrap_or_else(|| output_template.to_string_lossy().replace("%(ext)s", "mp4"));

  append_debug_log(
    &app,
    format!("video-downloader: download completed path={output_path}"),
  );

  Ok(VideoDownloadResult { output_path })
}

#[tauri::command]
fn list_tools(app: AppHandle) -> Result<Vec<Action>, String> {
  collect_actions(&app)
}

#[tauri::command(rename_all = "camelCase")]
fn run_tool(app: AppHandle, tool_id: String) -> Result<RunResult, String> {
  append_debug_log(&app, format!("run_tool: requested tool_id={tool_id}"));
  let actions = collect_actions(&app)?;
  append_debug_log(&app, format!("run_tool: collected actions={}", actions.len()));
  let action = actions
    .into_iter()
    .find(|action| action.id == tool_id)
    .ok_or_else(|| "Action no encontrada.".to_string())?;
  append_debug_log(
    &app,
    format!(
      "run_tool: matched id={} name={} version={} runtime={}",
      action.id, action.name, action.version, action.runtime.runtime_type
    ),
  );

  if action.runtime.runtime_type != "window" {
    if action.runtime.runtime_type == "overlay" {
      open_action_overlay(&app, &action)?;

      if let Some(launcher) = app.get_webview_window("main") {
        let _ = launcher.hide();
      }

      return Ok(RunResult {
        ok: true,
        message: format!("{} activa.", action.name),
      });
    }

    return Ok(RunResult {
      ok: false,
      message: format!("{} usa un runtime que LUMA aun no abre.", action.name),
    });
  }

  if action.id == "luma.action.copy-color" {
    activate_copy_color(&app)?;

    if let Some(launcher) = app.get_webview_window("main") {
      let _ = launcher.hide();
    }

    return Ok(RunResult {
      ok: true,
      message: "Gotero activo. Hace click en cualquier punto de la pantalla.".to_string(),
    });
  }

  if action.id == "luma.action.translate-this" {
    append_debug_log(&app, "run_tool: using dedicated Translate This window path");
    if let Some(launcher) = app.get_webview_window("main") {
      let _ = launcher.hide();
      append_debug_log(&app, "run_tool: launcher hidden for Translate This");
    }

    open_translate_this_window(&app)?;

    return Ok(RunResult {
      ok: true,
      message: "Translate This abierta.".to_string(),
    });
  }

  if action.id == "luma.action.contar-caracteres" {
    append_debug_log(&app, "run_tool: using dedicated ContarCaracteres window path");
    if let Some(launcher) = app.get_webview_window("main") {
      let _ = launcher.hide();
      append_debug_log(&app, "run_tool: launcher hidden for ContarCaracteres");
    }

    open_count_characters_window(&app)?;

    return Ok(RunResult {
      ok: true,
      message: "ContarCaracteres abierta.".to_string(),
    });
  }

  if action.id == "luma.action.merge-pdf" {
    append_debug_log(&app, "run_tool: using dedicated Merge PDF window path");
    if let Some(launcher) = app.get_webview_window("main") {
      let _ = launcher.hide();
      append_debug_log(&app, "run_tool: launcher hidden for Merge PDF");
    }

    open_merge_pdf_window(&app)?;

    return Ok(RunResult {
      ok: true,
      message: "Merge PDF abierta.".to_string(),
    });
  }

  if action.id == "luma.action.image-convert" {
    append_debug_log(&app, "run_tool: using dedicated Image Convert window path");
    if let Some(launcher) = app.get_webview_window("main") {
      let _ = launcher.hide();
      append_debug_log(&app, "run_tool: launcher hidden for Image Convert");
    }

    open_image_convert_window(&app)?;

    return Ok(RunResult {
      ok: true,
      message: "Image Convert abierta.".to_string(),
    });
  }

  if action.id == "luma.action.video-downloader" {
    append_debug_log(&app, "run_tool: using dedicated Video Downloader window path");
    if let Some(launcher) = app.get_webview_window("main") {
      let _ = launcher.hide();
      append_debug_log(&app, "run_tool: launcher hidden for Video Downloader");
    }

    open_video_downloader_window(&app)?;

    return Ok(RunResult {
      ok: true,
      message: "Video Downloader abierta.".to_string(),
    });
  }

  if let Some(launcher) = app.get_webview_window("main") {
    let _ = launcher.hide();
  }

  if let Err(error) = open_action_window(&app, &action) {
    let _ = show_launcher(&app);
    return Err(error);
  }

  Ok(RunResult {
    ok: true,
    message: format!("{} abierta.", action.name),
  })
}

#[tauri::command]
fn install_tool(app: AppHandle) -> Result<Option<InstallResult>, String> {
  let launcher = app.get_webview_window("main");
  if let Some(window) = launcher.as_ref() {
    let _ = window.hide();
  }

  let selected_path = rfd::FileDialog::new()
    .set_title("Instalar LUMA Action")
    .add_filter("LUMA Action", &["lm"])
    .pick_file();

  if let Some(window) = launcher.as_ref() {
    let _ = window.show();
    let _ = window.set_focus();
  }

  let Some(path) = selected_path else {
    return Ok(None);
  };

  install_action_bundle(&app, &path).map(Some)
}

#[tauri::command(rename_all = "camelCase")]
fn install_action_from_path(app: AppHandle, bundle_path: String) -> Result<InstallResult, String> {
  install_action_bundle(&app, Path::new(&bundle_path))
}

#[tauri::command]
fn hide_launcher(app: AppHandle) -> Result<(), String> {
  let window = app
    .get_webview_window("main")
    .ok_or_else(|| "No se encontro la ventana principal.".to_string())?;

  window.hide().map_err(|error| error.to_string())
}

#[tauri::command]
fn hide_current_window(window: WebviewWindow) -> Result<(), String> {
  window.hide().map_err(|error| error.to_string())
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
  let open_item = MenuItem::with_id(app, "open-luma", "Abrir LUMA", true, None::<&str>)?;
  let separator = PredefinedMenuItem::separator(app)?;
  let quit_item = MenuItem::with_id(app, "quit-luma", "Cerrar LUMA", true, None::<&str>)?;
  let menu = Menu::with_items(app, &[&open_item, &separator, &quit_item])?;

  let mut builder = TrayIconBuilder::with_id("luma")
    .tooltip("LUMA")
    .menu(&menu)
    .show_menu_on_left_click(false);

  if let Some(icon) = app.default_window_icon().cloned() {
    builder = builder.icon(icon);
  }

  builder
    .on_menu_event(|app, event| match event.id().as_ref() {
      "open-luma" => {
        let _ = show_launcher(app);
      }
      "quit-luma" => {
        force_quit_luma(app);
      }
      _ => {}
    })
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        let _ = show_launcher(tray.app_handle());
      }
    })
    .build(app)?;

  Ok(())
}

fn force_quit_luma(app: &AppHandle) {
  EXTRACT_TEXT_ACTIVE.store(false, Ordering::SeqCst);
  COPY_COLOR_ACTIVE.store(false, Ordering::SeqCst);

  for (_label, window) in app.webview_windows() {
    let _ = window.close();
  }

  app.exit(0);
  std::process::exit(0);
}

fn register_shortcuts(app: &AppHandle) -> Result<(), String> {
  let open_shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Backslash);
  let quit_shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Tab);
  app
    .global_shortcut()
    .register_multiple([open_shortcut, quit_shortcut])
    .map_err(|error| error.to_string())
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      let _ = show_launcher(app);
    }))
    .plugin(tauri_plugin_autostart::init(
      MacosLauncher::LaunchAgent,
      Some(vec![]),
    ))
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
          if event.state != ShortcutState::Pressed {
            return;
          }

          if shortcut.matches(Modifiers::SHIFT, Code::Tab) {
            force_quit_luma(app);
          } else if shortcut.matches(Modifiers::SHIFT, Code::Backslash) {
            let _ = show_launcher(app);
          }
        })
        .build(),
    )
    .setup(|app| {
      append_debug_log(&app.handle(), "setup: LUMA started");
      if let Ok(app_data_dir) = app.path().app_data_dir() {
        append_debug_log(
          &app.handle(),
          format!("setup: app_data_dir={}", app_data_dir.display()),
        );
      }

      if let Err(error) = app.autolaunch().enable() {
        eprintln!("Could not enable LUMA autostart: {error}");
        append_debug_log(&app.handle(), format!("setup: autostart failed: {error}"));
      }

      if let Err(error) = remove_legacy_bundled_actions(&app.handle()) {
        eprintln!("Could not remove legacy bundled Actions: {error}");
      }

      if let Err(error) = register_shortcuts(&app.handle()) {
        eprintln!("Could not register LUMA shortcuts: {error}");
      }

      create_tray(&app.handle())?;

      if let Err(error) = preload_declared_components(&app.handle()) {
        eprintln!("Could not preload LUMA components: {error}");
      }

      if let Err(error) = preload_extract_text_overlay(&app.handle()) {
        eprintln!("Could not preload Extract Text: {error}");
      }

      if let Some(window) = app.get_webview_window("main") {
        let blur_window = window.clone();
        window.on_window_event(move |event| {
          if matches!(event, tauri::WindowEvent::Focused(false)) {
            let _ = blur_window.hide();
          }
        });
      }

      if let Some(window) = app.get_webview_window("luma-translate-this") {
        append_debug_log(&app.handle(), "setup: translate-this predeclared window found");
        let log_app = app.handle().clone();
        let translate_window = window.clone();
        window.on_window_event(move |event| {
          append_debug_log(&log_app, format!("translate-this: window event: {event:?}"));
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = translate_window.hide();
            append_debug_log(&log_app, "translate-this: close requested intercepted and hidden");
          }
        });
      } else {
        append_debug_log(&app.handle(), "setup: translate-this predeclared window missing");
      }

      if let Some(window) = app.get_webview_window("luma-contar-caracteres") {
        append_debug_log(&app.handle(), "setup: contar-caracteres predeclared window found");
        let log_app = app.handle().clone();
        let count_window = window.clone();
        window.on_window_event(move |event| {
          append_debug_log(&log_app, format!("contar-caracteres: window event: {event:?}"));
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = count_window.hide();
            append_debug_log(
              &log_app,
              "contar-caracteres: close requested intercepted and hidden",
            );
          }
        });
      } else {
        append_debug_log(&app.handle(), "setup: contar-caracteres predeclared window missing");
      }

      if let Some(window) = app.get_webview_window("luma-merge-pdf") {
        append_debug_log(&app.handle(), "setup: merge-pdf predeclared window found");
        let log_app = app.handle().clone();
        let merge_window = window.clone();
        window.on_window_event(move |event| {
          append_debug_log(&log_app, format!("merge-pdf: window event: {event:?}"));
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = merge_window.hide();
            append_debug_log(&log_app, "merge-pdf: close requested intercepted and hidden");
          }
        });
      } else {
        append_debug_log(&app.handle(), "setup: merge-pdf predeclared window missing");
      }

      if let Some(window) = app.get_webview_window("luma-image-convert") {
        append_debug_log(&app.handle(), "setup: image-convert predeclared window found");
        let log_app = app.handle().clone();
        let convert_window = window.clone();
        window.on_window_event(move |event| {
          append_debug_log(&log_app, format!("image-convert: window event: {event:?}"));
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = convert_window.hide();
            append_debug_log(&log_app, "image-convert: close requested intercepted and hidden");
          }
        });
      } else {
        append_debug_log(&app.handle(), "setup: image-convert predeclared window missing");
      }

      if let Some(window) = app.get_webview_window("luma-video-downloader") {
        append_debug_log(&app.handle(), "setup: video-downloader predeclared window found");
        let log_app = app.handle().clone();
        let video_window = window.clone();
        window.on_window_event(move |event| {
          append_debug_log(&log_app, format!("video-downloader: window event: {event:?}"));
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = video_window.hide();
            append_debug_log(
              &log_app,
              "video-downloader: close requested intercepted and hidden",
            );
          }
        });
      } else {
        append_debug_log(&app.handle(), "setup: video-downloader predeclared window missing");
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      list_tools,
      run_tool,
      install_tool,
      install_action_from_path,
      get_current_window_action_html,
      get_virtual_screen_bounds,
      capture_screen_region,
      luma_debug_log,
      ocr_screen_region,
      translate_text,
      translate_text_to_spanish,
      merge_pdf_pick_files,
      merge_pdf_export,
      image_convert_pick_input,
      image_convert_export,
      video_downloader_preview,
      video_downloader_download,
      translate_image_from_screen_region,
      show_current_window_panel,
      extract_text_from_screen_region,
      write_clipboard_text,
      sample_copy_color,
      finish_copy_color,
      cancel_copy_color,
      hide_launcher,
      hide_current_window
    ])
    .run(tauri::generate_context!())
    .expect("error while running LUMA");
}
