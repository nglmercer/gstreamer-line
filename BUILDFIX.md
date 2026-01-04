# Build Fix for Bazzite

## Problem
The build was failing because the linker could not find the GStreamer libraries (`-lgstapp-1.0`, `-lgstbase-1.0`, `-lgstreamer-1.0`).

## Root Cause
On Bazzite, GStreamer is installed in `/usr/lib64/` but:
1. pkg-config files are not installed, so the Rust GStreamer crates cannot automatically detect the libraries
2. The libraries are versioned (e.g., `libgstapp-1.0.so.0`) but the linker expects unversioned names (e.g., `libgstapp-1.0.so`)

## Solution
The fix involved three changes:

### 1. Updated `build.rs`
Added logic to detect when pkg-config is not available and manually configure library linking:

```rust
fn main() {
  napi_build::setup();
  println!("cargo:rerun-if-changed=build.rs");

  if let Ok(_) = pkg_config::probe_library("gstreamer-1.0") {
    // pkg-config found, the crates will handle linking
    println!("cargo:rustc-link-lib=gstapp-1.0");
    println!("cargo:rustc-link-lib=gstbase-1.0");
    println!("cargo:rustc-link-lib=gstreamer-1.0");
  } else {
    // Fallback: manually link against system GStreamer libraries
    println!("cargo:warning=gstreamer-1.0 not found via pkg-config, using system libraries");
    println!("cargo:rustc-link-search=native=gst-lib");
    println!("cargo:rustc-link-lib=gstapp-1.0");
    println!("cargo:rustc-link-lib=gstbase-1.0");
    println!("cargo:rustc-link-lib=gstreamer-1.0");
  }
}
```

### 2. Updated `Cargo.toml`
Added `pkg-config` as a build dependency:

```toml
[build-dependencies]
napi-build = "2"
pkg-config = "0.3"
```

### 3. Created Library Symlinks
Created a `gst-lib/` directory containing symlinks from unversioned library names to the versioned system libraries:

```bash
gst-lib/
  libgstapp-1.0.so -> /usr/lib64/libgstapp-1.0.so.0
  libgstbase-1.0.so -> /usr/lib64/libgstbase-1.0.so.0
  libgstreamer-1.0.so -> /usr/lib64/libgstreamer-1.0.so.0
```

### 4. Updated `.gitignore`
Added `gst-lib/` to prevent committing the symlinks (since they're system-specific).

## Result
The build now completes successfully on Bazzite using the system GStreamer installation:

```bash
$ bun run build
$ napi build --platform --release
   Compiling napi-package-template v0.1.0
warning: napi-package-template@0.1.0: gstreamer-1.0 not found via pkg-config, using system libraries
    Finished `release` profile [optimized] target(s) in 7.03s
```

## Notes
- This fix allows the project to build on systems where GStreamer is installed but pkg-config files are not available
- The `gst-lib/` directory is ignored by git and should be recreated by each developer on their system if needed
- On systems with proper pkg-config setup, the build will use pkg-config automatically
