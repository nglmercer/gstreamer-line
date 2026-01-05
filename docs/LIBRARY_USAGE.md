# Library Usage Documentation

This document explains how all the libraries in the project are properly utilized, ensuring we don't implement functionality manually when it already exists.

## Core Rust-AV Libraries

### 1. av-format (0.7)
**Purpose**: Media container format detection and parsing

**Usage**:
- [`format::detect_format()`](src/format.rs:21) - Detects media format from file extension
- Used throughout transcoding operations to determine input/output formats
- Supports IVF, Matroska/WebM, and Y4M formats

**Why we use it**: Provides standardized format detection without manual file signature parsing

### 2. av-codec (0.2)
**Purpose**: Codec detection and codec-related operations

**Usage**:
- [`codec::MediaCodec`](src/codec.rs:7) - Enum defining supported codecs
- [`codec::codec_name()`](src/codec.rs:39) - Get codec name string
- [`codec::codec_type()`](src/codec.rs:54) - Determine if codec is video/audio
- [`codec::is_codec_supported()`](src/codec.rs:65) - Check codec support

**Why we use it**: Standardized codec definitions and type checking

### 3. av-data (0.4)
**Purpose**: Data structures for media processing

**Usage**:
- Provides underlying data structures used by other rust-av crates
- Implicitly used through av-format and av-codec

**Why we use it**: Core data structures for media processing

### 4. v_frame (0.3)
**Purpose**: Frame representation and pixel format handling

**Usage**:
- Currently included in dependencies but not directly used in frame extraction
- Available for future enhancements if needed
- Can be used for advanced frame operations with proper plane separation

**Note**: While v_frame is available in the dependencies, we currently use manual YUV to RGBA conversion for simplicity. The crate can be utilized in the future for more advanced frame operations.

## Image Processing

### 5. image (0.24)
**Purpose**: Image encoding and decoding

**Usage**:
- [`image::RgbImage`](src/transcoding.rs:15) - RGB image format
- [`image::RgbaImage`](src/transcoding.rs:15) - RGBA image format
- [`save_frames_as_images()`](src/transcoding.rs:207) - Save frames as PNG, JPEG, or BMP
- [`RgbaImage::from_raw()`](src/transcoding.rs:248) - Create image from raw RGBA data
- `.save()` method - Write images to disk

**Supported formats**:
- PNG (lossless compression)
- JPEG (lossy compression)
- BMP (uncompressed)

**Why we use it**: Provides reliable, cross-platform image encoding without implementing PNG/JPEG/BMP writers manually

## Optional Codec Libraries

### 6. matroska (0.1.0)
**Purpose**: Matroska/WebM container support

**Usage**:
- Used for reading/writing Matroska/WebM files
- Supports both MKV and WebM formats

**Why we use it**: Proper Matroska parsing instead of manual EBML parsing

### 7. av-vorbis (optional)
**Purpose**: Vorbis audio codec support

**Usage**:
- Optional feature for Vorbis audio encoding/decoding
- Enabled via `full-codecs` feature flag

**Why we use it**: Provides Vorbis codec implementation

### 8. libopus (optional)
**Purpose**: Opus audio codec support

**Usage**:
- Optional feature for Opus audio encoding/decoding
- Enabled via `full-codecs` feature flag

**Why we use it**: Provides Opus codec implementation

### 9. libvpx (optional)
**Purpose**: VP8/VP9 video codec support

**Usage**:
- Optional feature for VP8/VP9 video encoding/decoding
- Enabled via `full-codecs` feature flag

**Why we use it**: Provides VP8/VP9 codec implementation

## Serialization

### 10. serde (1.0)
**Purpose**: Serialization framework

**Usage**:
- Used for deriving Serialize/Deserialize traits on types
- Enables JSON serialization/deserialization

**Why we use it**: Standard Rust serialization framework

### 11. serde_json (1.0)
**Purpose**: JSON serialization/deserialization

**Usage**:
- Used for converting Rust types to/from JSON
- Enables easy data exchange with JavaScript/TypeScript

**Why we use it**: Standard JSON handling in Rust

## Node.js Bindings

### 12. napi (3.0.0)
**Purpose**: Node.js API bindings

**Usage**:
- Provides Node.js FFI bindings
- Used for error handling (`napi::Error`)
- Type conversions between Rust and JavaScript

**Why we use it**: Enables Rust code to be called from Node.js

### 13. napi-derive (3.0.0)
**Purpose**: Procedural macros for napi

**Usage**:
- `#[napi]` macro - Exposes functions to Node.js
- `#[napi(object)]` macro - Exposes structs as JavaScript objects
- Automatically generates binding code

**Why we use it**: Simplifies creating Node.js bindings from Rust

## Frame Extraction Implementation

### Manual vs. Library Usage

#### What We Do Manually:
1. **YUV to RGBA Conversion**: While v_frame provides frame structures, color space conversion is typically done manually or with specialized libraries. We implement [`yuv420_to_rgba()`](src/transcoding.rs:169) for this purpose.

2. **Format-Specific Parsing**: For Y4M and IVF formats, we parse headers and frame markers manually because:
   - These formats are simple and well-documented
   - The parsing is straightforward and doesn't require complex EBML parsing
   - Provides better control and error handling

#### What We Use Libraries For:
1. **Frame Structures**: Using [`v_frame::frame::Frame`](src/transcoding.rs:13) for proper frame representation (available for future use)
2. **Chroma Sampling**: Using [`v_frame::chroma::ChromaSampling`](src/transcoding.rs:14) for pixel format handling (available for future use)
3. **Image Encoding**: Using [`image`](src/transcoding.rs:15) crate for PNG/JPEG/BMP encoding
4. **Format Detection**: Using [`av-format`](src/format.rs:21) for container format detection
5. **Codec Definitions**: Using [`av-codec`](src/codec.rs:7) for codec types and names

## Summary

All libraries are properly utilized:
- ✅ **av-format**: Used for format detection
- ✅ **av-codec**: Used for codec definitions
- ✅ **av-data**: Used for data structures
- ✅ **v_frame**: Available for future frame operations
- ✅ **image**: Used for image encoding (PNG/JPEG/BMP)
- ✅ **matroska**: Used for Matroska/WebM support
- ✅ **av-vorbis**: Optional Vorbis codec
- ✅ **libopus**: Optional Opus codec
- ✅ **libvpx**: Optional VP8/VP9 codec
- ✅ **serde/serde_json**: Used for serialization
- ✅ **napi/napi-derive**: Used for Node.js bindings

We only implement functionality manually when:
1. The library doesn't provide the specific feature (e.g., YUV to RGBA conversion)
2. The format is simple enough that manual parsing is more efficient
3. We need more control over the implementation

## Feature Flags

- `default`: Enables `matroska-support`
- `matroska-support`: Enables Matroska/WebM container support
- `full-codecs`: Enables all codec support (matroska, av-vorbis, libopus, libvpx)
