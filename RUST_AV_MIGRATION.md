# Rust-AV Migration Summary

## Overview

This document summarizes the migration from FFmpeg to Rust-AV ecosystem for the gstreamer-line project.

## Changes Made

### 1. Cargo.toml Updates

- **Removed**: `ffmpeg-next = "7.0"` dependency
- **Added**: Rust-AV ecosystem crates:
  - `av-data = "0.4"` - Multimedia data structures (Frame, Packet)
  - `av-format = "0.7"` - Format handling (muxers/demuxers)
  - `v_frame = "0.3"` - Video frame structures
- **Updated**: Package name from `ffmpeg-kit` to `rust-av-kit`
- **Added**: `rlib` crate type for testing support

### 2. src/lib.rs Updates

- **Replaced**: FFmpeg initialization with Rust-AV (no explicit initialization needed)
- **Updated**: Function signatures to use `Result<T, napi::Error>` instead of `Result<T, String>`
- **Implemented**: Placeholder functions for core functionality:
  - `get_media_info()` - Extract media file information
  - `transcode()` - Media transcoding
  - `get_supported_formats()` - List supported container formats
  - `get_supported_codecs()` - List supported codecs
  - `get_supported_pixel_formats()` - List pixel formats
  - `get_supported_sample_formats()` - List sample formats

### 3. Tests Created

#### Rust Tests (tests/integration_test.rs)
- 7 unit tests covering:
  - Supported formats, codecs, pixel formats, sample formats
  - Data structure validation (StreamInfo, FormatInfo, MediaInfo, etc.)
  - Cloning functionality for all structs

#### Bun Tests (__test__/rust-av.test.ts)
- 14 integration tests covering:
  - API function behavior
  - Error handling for invalid inputs
  - Data structure validation
  - Format and codec combinations

## Rust-AV Ecosystem

Based on the technical report provided, Rust-AV is a modular ecosystem consisting of:

### Core Crates
- **av-data**: Data structures for multimedia (Frame, Packet)
- **av-format**: Format handling with muxer/demuxer traits
- **v_frame**: Video frame representation and manipulation
- **av-decoders**: Decoder abstraction layer (optional FFmpeg backend)
- **av-encoders**: Encoder implementations

### Supported Formats
- IVF (Indeo Video Format) - for VP9/AV1 bitstreams
- Matroska/WebM - native implementation
- Y4M - uncompressed video format
- MP4 - via external crate (mp4-rs)

### Supported Codecs
- **Video**: AV1, VP8, VP9, H.264, H.265
- **Audio**: Opus, Vorbis, PCM
- **Note**: H.264/H.265 support requires FFmpeg backend via av-decoders

## Current Implementation Status

### ✅ Completed
- [x] Project structure updated
- [x] Dependencies migrated to Rust-AV ecosystem
- [x] NAPI bindings compiled successfully
- [x] Rust tests passing (7/7)
- [x] Bun tests passing (14/14)

### 🚧 TODO (Placeholder Implementation)

The following functions have placeholder implementations that need to be completed:

#### get_media_info()
```rust
// TODO: Implement using av-format
// 1. Open file using appropriate demuxer
// 2. Read format information
// 3. Extract stream details
// 4. Parse codec information
```

#### transcode()
```rust
// TODO: Implement using av-format, av-data, v_frame
// 1. Open input with demuxer
// 2. Read packets from input
// 3. Decode packets to frames
// 4. Apply filters if specified
// 5. Encode frames using encoders
// 6. Write packets to output with muxer
```

## Build and Test Commands

### Build Rust library
```bash
cargo build --release
```

### Run Rust tests
```bash
cargo test
```

### Build NAPI bindings
```bash
bun run build
```

### Run Bun tests
```bash
bun test
```

## Architecture Notes

### Modular Design
Rust-AV's modular design allows:
- **Zero-copy operations** between components
- **Selective dependency inclusion** via features
- **Native implementations** for modern formats (AV1, VP9)
- **FFmpeg integration** for legacy format support (optional)

### Performance Considerations
- Native Rust implementations provide memory safety
- SIMD optimizations available in hot paths
- Hardware acceleration support via external crates (nvidia-video-codec-rs)

## Next Steps

1. **Implement actual media info extraction** using av-format
2. **Implement transcoding pipeline** with av-decoders/av-encoders
3. **Add filter support** using v_frame transformations
4. **Add progress callbacks** for long-running operations
5. **Consider hardware acceleration** via platform-specific crates

## References

- [Rust-AV GitHub Organization](https://github.com/rust-av)
- [av-data Documentation](https://docs.rs/av-data)
- [av-format Documentation](https://docs.rs/av-format)
- [v_frame Documentation](https://docs.rs/v_frame)
- [rav1e](https://github.com/xiph/rav1e) - AV1 encoder reference
- [dav1d-rs](https://code.videolan.org/videolan/dav1d) - AV1 decoder bindings
