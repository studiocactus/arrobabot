fn main() {
 tauri_build::build();
 // Tauri embeds this dependency in the application resources. Unit-test binaries
 // also link the native dialog backend and need the same Common Controls v6.
 if std::env::var("CARGO_CFG_TARGET_OS").as_deref()==Ok("windows") {
  println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
  println!("cargo:rustc-link-arg-bin=botlive=/MANIFEST:NO");
  println!("cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
 }
}

