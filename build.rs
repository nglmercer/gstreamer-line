fn main() {
  // Setup napi build
  napi_build::setup();

  // Ensure GStreamer libraries are properly linked
  println!("cargo:rerun-if-changed=build.rs");

  // Try to use pkg-config first
  if let Ok(_) = pkg_config::probe_library("gstreamer-1.0") {
    // pkg-config found, the crates will handle linking
    println!("cargo:rustc-link-lib=gstapp-1.0");
    println!("cargo:rustc-link-lib=gstbase-1.0");
    println!("cargo:rustc-link-lib=gstreamer-1.0");
  } else {
    // Fallback: manually link against system GStreamer libraries
    // This is needed for systems like Bazzite where pkg-config files are not installed
    println!("cargo:warning=gstreamer-1.0 not found via pkg-config, using system libraries");

    // Add library search path for local symlinks (pointing to system libraries)
    println!("cargo:rustc-link-search=native=gst-lib");

    // Link against GStreamer libraries
    println!("cargo:rustc-link-lib=gstapp-1.0");
    println!("cargo:rustc-link-lib=gstbase-1.0");
    println!("cargo:rustc-link-lib=gstreamer-1.0");
  }
}
