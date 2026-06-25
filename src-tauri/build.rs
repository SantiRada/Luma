fn main() {
  println!("cargo:rerun-if-changed=../src/renderer/index.html");
  println!("cargo:rerun-if-changed=../src/renderer/copy-color-overlay.html");
  println!("cargo:rerun-if-changed=../src/renderer/copy-color-overlay.js");
  println!("cargo:rerun-if-changed=../src/renderer/copy-color-overlay.css");
  println!("cargo:rerun-if-changed=../src/renderer/renderer.js");
  println!("cargo:rerun-if-changed=../src/renderer/styles.css");
  tauri_build::build()
}
