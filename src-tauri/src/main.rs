use serde::{Deserialize, Serialize};
use std::{
  collections::HashSet,
  fs,
  fs::File,
  io::Read,
  path::{Path, PathBuf},
  sync::atomic::{AtomicBool, AtomicU64, Ordering},
  thread,
  time::Duration,
};
use tauri::{
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Url, WebviewUrl, WebviewWindow,
  WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{
  Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};
use zip::ZipArchive;
use base64::{engine::general_purpose, Engine as _};

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

static COPY_COLOR_ACTIVE: AtomicBool = AtomicBool::new(false);
static COPY_COLOR_SESSION: AtomicU64 = AtomicU64::new(0);

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
  let mut seen_ids = actions
    .iter()
    .map(|action| action.id.clone())
    .collect::<HashSet<_>>();

  if let Ok(dev_dir) = development_actions_dir(app) {
    for action in read_actions_from_dir(&dev_dir, "development") {
      if seen_ids.insert(action.id.clone()) {
        actions.push(action);
      }
    }
  }

  actions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  Ok(actions)
}

fn center_window_on_cursor(window: &WebviewWindow) -> tauri::Result<()> {
  let size = window.outer_size()?;
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
    let x = position.x + ((monitor_size.width as i32 - size.width as i32) / 2);
    let y = position.y + ((monitor_size.height as i32 - size.height as i32) / 2);
    window.set_position(PhysicalPosition::new(x, y))?;
    window.set_size(PhysicalSize::new(size.width, size.height))?;
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

fn install_action_bundle(app: &AppHandle, bundle_path: &Path) -> Result<Action, String> {
  validate_lm_path(bundle_path)?;
  let mut action = read_action_manifest_from_bundle(bundle_path)?;

  if !action.id.starts_with("luma.action.") {
    return Err("El id de la Action debe empezar con luma.action.".to_string());
  }

  let action_dir = app_actions_dir(app)?.join(sanitize_action_id(&action.id));
  extract_action_bundle(bundle_path, &action_dir)?;

  action.source = ActionSource {
    kind: "installed".to_string(),
    path: action_dir.to_string_lossy().to_string(),
    manifest_path: action_dir.join("manifest.json").to_string_lossy().to_string(),
  };

  Ok(action)
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

fn inline_action_html(entry_path: &Path) -> Result<String, String> {
  let action_dir = entry_path
    .parent()
    .ok_or_else(|| "No se pudo resolver la carpeta de la Action.".to_string())?;
  let mut html = fs::read_to_string(entry_path).map_err(|error| error.to_string())?;

  for stylesheet_name in ["styles.css"] {
    let stylesheet_path = action_dir.join(stylesheet_name);
    if stylesheet_path.exists() {
      let css = fs::read_to_string(&stylesheet_path).map_err(|error| error.to_string())?;
      html = html.replace(
        &format!(r#"<link rel="stylesheet" href="./{}" />"#, stylesheet_name),
        &format!("<style>\n{}\n</style>", css),
      );
    }
  }

  for script_name in ["action.js"] {
    let script_path = action_dir.join(script_name);
    if script_path.exists() {
      let js = fs::read_to_string(&script_path).map_err(|error| error.to_string())?;
      html = html.replace(
        &format!(r#"<script src="./{}"></script>"#, script_name),
        &format!("<script>\n{}\n</script>", js),
      );
    }
  }

  Ok(html)
}

fn data_url_for_action(entry_path: &Path) -> Result<Url, String> {
  let html = inline_action_html(entry_path)?;
  let encoded = general_purpose::STANDARD.encode(html.as_bytes());
  Url::parse(&format!("data:text/html;base64,{encoded}")).map_err(|error| error.to_string())
}

fn file_url_for_action(entry_path: &Path) -> Result<Url, String> {
  Url::from_file_path(entry_path)
    .map_err(|_| format!("No se pudo abrir la entrada de la Action: {}", entry_path.display()))
}

fn copy_color_hud_url() -> Result<Url, String> {
  let html = r##"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <style>
      * { box-sizing: border-box; }
      html, body { width: 100%; height: 100%; margin: 0; overflow: hidden; background: transparent; font-family: "DM Sans", "Inter", "Segoe UI", sans-serif; }
      body { display: grid; place-items: center; }
      .hud { display: grid; grid-template-columns: 64px 1fr; gap: 12px; align-items: center; width: 292px; min-height: 78px; border: 1px solid rgba(255,255,255,.12); border-radius: 10px; background: #2a2a2a; box-shadow: 0 10px 28px rgba(0,0,0,.38); padding: 10px; color: #fff; }
      .grid { display: grid; width: 56px; height: 56px; overflow: hidden; grid-template-columns: repeat(9, 1fr); grid-template-rows: repeat(9, 1fr); border-radius: 6px; background: #111; box-shadow: inset 0 0 0 1px rgba(255,255,255,.08); }
      .pixel.center { box-shadow: inset 0 0 0 1px #fff, inset 0 0 0 2px #111; z-index: 1; }
      .copy { display: grid; gap: 6px; }
      .hex-row { display: flex; align-items: center; gap: 8px; }
      .swatch { width: 15px; height: 15px; border: 1px solid rgba(255,255,255,.14); border-radius: 4px; background: #fff; }
      strong { font-size: 13px; letter-spacing: 0; }
      p { margin: 0; color: rgba(255,255,255,.68); font-size: 13px; }
      .dropper { color: rgba(255,255,255,.8); margin-right: 4px; }
    </style>
  </head>
  <body>
    <section class="hud">
      <div id="grid" class="grid"></div>
      <div class="copy">
        <div class="hex-row"><span id="swatch" class="swatch"></span><strong id="hex">#FFFFFF</strong></div>
        <p><span class="dropper">⌁</span> Click to Copy</p>
      </div>
    </section>
    <script>
      const grid = document.querySelector('#grid');
      const hex = document.querySelector('#hex');
      const swatch = document.querySelector('#swatch');
      for (let i = 0; i < 81; i += 1) {
        const pixel = document.createElement('span');
        pixel.className = i === 40 ? 'pixel center' : 'pixel';
        grid.appendChild(pixel);
      }
      window.__setSample = (nextHex, pixels) => {
        hex.textContent = nextHex;
        swatch.style.background = nextHex;
        pixels.forEach((color, index) => {
          grid.children[index].style.background = color;
        });
      };
    </script>
  </body>
</html>"##;
  let encoded = general_purpose::STANDARD.encode(html.as_bytes());
  Url::parse(&format!("data:text/html;base64,{encoded}")).map_err(|error| error.to_string())
}

fn open_action_window(app: &AppHandle, action: &Action) -> Result<(), String> {
  let entry_path = action_entry_path(action)?;
  let label = action_window_label(&action.id);

  if let Some(window) = app.get_webview_window(&label) {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    return Ok(());
  }

  let url = file_url_for_action(&entry_path)?;
  let width = action.runtime.width.unwrap_or(520) as f64;
  let height = action.runtime.height.unwrap_or(360) as f64;

  let window = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
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

  Ok(())
}

fn open_action_overlay(app: &AppHandle, action: &Action) -> Result<(), String> {
  let entry_path = action_entry_path(action)?;
  let label = action_window_label(&action.id);
  let url = file_url_for_action(&entry_path)?;
  let (screen_x, screen_y, screen_width, screen_height) = virtual_screen_bounds();

  if let Some(window) = app.get_webview_window(&label) {
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
    return Ok(());
  }

  let window = WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
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

  window.set_focus().map_err(|error| error.to_string())?;
  window
    .emit("luma-overlay-start", serde_json::json!({ "x": screen_x, "y": screen_y }))
    .map_err(|error| error.to_string())?;

  Ok(())
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

#[tauri::command]
fn list_tools(app: AppHandle) -> Result<Vec<Action>, String> {
  collect_actions(&app)
}

#[tauri::command(rename_all = "camelCase")]
fn run_tool(app: AppHandle, tool_id: String) -> Result<RunResult, String> {
  let actions = collect_actions(&app)?;
  let action = actions
    .into_iter()
    .find(|action| action.id == tool_id)
    .ok_or_else(|| "Action no encontrada.".to_string())?;

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

  open_action_window(&app, &action)?;

  if let Some(launcher) = app.get_webview_window("main") {
    let _ = launcher.hide();
  }

  Ok(RunResult {
    ok: true,
    message: format!("{} abierta.", action.name),
  })
}

#[tauri::command]
fn install_tool(app: AppHandle) -> Result<Option<Action>, String> {
  let Some(path) = rfd::FileDialog::new()
    .set_title("Instalar LUMA Action")
    .add_filter("LUMA Action", &["lm"])
    .pick_file()
  else {
    return Ok(None);
  };

  install_action_bundle(&app, &path).map(Some)
}

#[tauri::command(rename_all = "camelCase")]
fn install_action_from_path(app: AppHandle, bundle_path: String) -> Result<Action, String> {
  install_action_bundle(&app, Path::new(&bundle_path))
}

#[tauri::command]
fn hide_launcher(app: AppHandle) -> Result<(), String> {
  let window = app
    .get_webview_window("main")
    .ok_or_else(|| "No se encontro la ventana principal.".to_string())?;

  window.hide().map_err(|error| error.to_string())
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
  let mut builder = TrayIconBuilder::with_id("luma").tooltip("LUMA");

  if let Some(icon) = app.default_window_icon().cloned() {
    builder = builder.icon(icon);
  }

  builder
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

fn register_shortcut(app: &AppHandle) -> Result<(), String> {
  let shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Backslash);
  app
    .global_shortcut()
    .register(shortcut)
    .map_err(|error| error.to_string())
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      let _ = show_launcher(app);
    }))
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
          if event.state == ShortcutState::Pressed {
            let _ = show_launcher(app);
          }
        })
        .build(),
    )
    .setup(|app| {
      if let Err(error) = register_shortcut(&app.handle()) {
        eprintln!("Could not register LUMA shortcut: {error}");
      }

      create_tray(&app.handle())?;

      if let Some(window) = app.get_webview_window("main") {
        let blur_window = window.clone();
        window.on_window_event(move |event| {
          if matches!(event, tauri::WindowEvent::Focused(false)) {
            let _ = blur_window.hide();
          }
        });
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      list_tools,
      run_tool,
      install_tool,
      install_action_from_path,
      get_virtual_screen_bounds,
      capture_screen_region,
      write_clipboard_text,
      sample_copy_color,
      finish_copy_color,
      cancel_copy_color,
      hide_launcher
    ])
    .run(tauri::generate_context!())
    .expect("error while running LUMA");
}
