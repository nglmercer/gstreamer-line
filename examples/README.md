# Rust-AV Kit Examples

This directory contains examples demonstrating how to use the Rust-AV Kit library for media transcoding, format transformation, and media info extraction.

## Quick Start

### 1. Fetch Test Videos

Before running the examples, you can download sample videos in different formats:

```bash
bun run videos:fetch
```

This will download sample videos from public sources (Big Buck Bunny, Sintel) in Y4M format and convert them to IVF and Matroska (MKV) formats if FFmpeg is available.

### 2. Run the Basic Usage Example

```bash
bun run examples/basic_usage.ts
```

The example will use the fetched videos if available, or create minimal synthetic test files as fallback.

### 3. Clean Up Test Videos

To remove all test videos:

```bash
bun run videos:clean
```

### 4. List Available Test Videos

To see which test videos are available:

```bash
bun run videos:list
```

## Available Scripts

| Command | Description |
|---------|-------------|
| `bun run videos:fetch` | Download sample videos from public sources |
| `bun run videos:clean` | Remove all test videos from the test_files directory |
| `bun run videos:list` | List available test videos and their sizes |

## Video Fetching Script

The [`fetch_test_videos.ts`](fetch_test_videos.ts) script provides the following features:

### Supported Video Formats

- **Y4M** - Raw YUV video format (downloaded directly)
- **IVF** - VP8/VP9 container format (converted from Y4M using FFmpeg)
- **MKV** - Matroska container format (converted from Y4M using FFmpeg)

### Video Sources

The script downloads sample videos from reliable GitHub-hosted sources:

- **sample_320x240.y4m** - 320x240 Y4M video
- **sample_640x360.y4m** - 640x360 Y4M video
- **sample_320x240.ivf** - 320x240 IVF video (VP8)
- **sample_small.mkv** - Small MKV test file

### Requirements

- **Node.js** >= 10
- **Bun** (for running TypeScript files)
- **FFmpeg** (optional - videos are downloaded directly in their target formats)

To install FFmpeg:
- **Linux**: `sudo apt-get install ffmpeg` or `sudo yum install ffmpeg`
- **macOS**: `brew install ffmpeg`
- **Windows**: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

### Script Commands

```bash
# Fetch videos (default command)
bun run examples/fetch_test_videos.ts fetch

# Clean up videos
bun run examples/fetch_test_videos.ts clean

# List videos
bun run examples/fetch_test_videos.ts list

# Show help
bun run examples/fetch_test_videos.ts help
```

## Basic Usage Example

The [`basic_usage.ts`](basic_usage.ts) file demonstrates:

1. **Getting Supported Formats and Codecs** - Query available formats, codecs, pixel formats, and sample formats
2. **Getting Media Information** - Extract metadata from video files
3. **Format Transformation** - Convert between different video formats
4. **Advanced Transcoding** - Transcode with codec options (resolution, bitrate, etc.)
5. **Video Filters** - Apply filters like scaling during transcoding
6. **Error Handling** - Proper error handling for missing files and invalid operations

### Example Output

```
=== Example 1: Get Supported Formats and Codecs ===

Supported formats: ['ivf', 'matroska', 'y4m']
Supported codecs: ['av1', 'vp8', 'vp9', 'h264', 'h265']
Supported pixel formats: ['yuv420p', 'yuv422p', 'yuv444p', 'rgb24', 'bgr24', 'rgba']
Supported sample formats: ['u8', 's16', 's32', 'f32']

=== Example 2: Get Media Information ===

Media Info: {
  "format": {
    "name": "y4m",
    "longName": "YUV4MPEG2",
    "duration": 0.03,
    "bitRate": 12345678,
    "startTime": 0,
    "nbStreams": 1
  },
  "streams": [...]
}
```

## Test Files Directory

The `test_files/` directory contains:

- **Downloaded videos** - Sample videos fetched from public sources
- **Generated videos** - Output files created during example execution

Generated files are automatically cleaned up after running the examples, but downloaded videos are preserved for reuse.

## Troubleshooting

### FFmpeg Not Found

If you see the warning "FFmpeg not found", install FFmpeg to enable format conversion:

```bash
# Linux (Debian/Ubuntu)
sudo apt-get install ffmpeg

# macOS
brew install ffmpeg

# Windows
# Download from https://ffmpeg.org/download.html
```

### Download Failures

If video downloads fail, check your internet connection and ensure the Xiph.org servers are accessible. The script will skip failed downloads and continue with available videos.

### Permission Errors

If you encounter permission errors when creating files in the test_files directory, ensure you have write permissions:

```bash
chmod +w examples/test_files
```

## Additional Resources

- [Rust-AV Kit Documentation](../README.md)
- [NAPI-RS Documentation](https://napi.rs/)
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)
- [Xiph.org Test Media](https://media.xiph.org/video/derf/)
