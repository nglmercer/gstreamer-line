# Video Generation with GstKit

This guide explains how to generate test videos using the GstKit library and answers common questions about video generation capabilities.

## Table of Contents

1. [Can GstKit Generate Test Videos?](#can-gstkit-generate-test-videos)
2. [Available Test Patterns](#available-test-patterns)
3. [Video Formats and Encoders](#video-formats-and-encoders)
4. [Examples](#examples)
5. [FFmpeg vs GStreamer](#ffmpeg-vs-gstreamer)
6. [Requirements](#requirements)

---

## Can GstKit Generate Test Videos?

**Yes!** GstKit can generate test videos using GStreamer's built-in test sources and encoders. GStreamer provides several test pattern generators that can create video content without needing external video files.

### Key Features:

- **Built-in test sources**: No need for external video files
- **Multiple patterns**: Various test patterns available
- **Custom data**: Generate video from your own data using AppSrc
- **Audio support**: Include audio tracks with test audio sources
- **Multiple formats**: MP4, WebM, AVI, and more
- **Real-time encoding**: Encode on-the-fly as the video is generated

---

## Available Test Patterns

GStreamer's `videotestsrc` element provides several test patterns:

| Pattern | Description |
|---------|-------------|
| `snow` | Random noise (static) |
| `black` | Black screen |
| `white` | White screen |
| `red` | Red screen |
| `green` | Green screen |
| `blue` | Blue screen |
| `checkers-1` | Checkerboard pattern (1) |
| `checkers-2` | Checkerboard pattern (2) |
| `checkers-4` | Checkerboard pattern (4) |
| `checkers-8` | Checkerboard pattern (8) |
| `circular` | Circular pattern |
| `gradient` | Color gradient |
| `colors` | Color bars (SMPTE style) |
| `smpte` | SMPTE color bars |
| `ball` | Moving ball animation |
| `smpte75` | SMPTE 75% color bars |
| `zone-plate` | Zone plate test pattern |

### Audio Test Patterns

The `audiotestsrc` element provides several audio waveforms:

| Wave | Description |
|------|-------------|
| `sine` | Sine wave |
| `square` | Square wave |
| `saw` | Sawtooth wave |
| `triangle` | Triangle wave |
| `silence` | Silence |
| `white-noise` | White noise |
| `pink-noise` | Pink noise |
| `sine-table` | Sine wave from table |
| `ticks` | Periodic ticks |

---

## Video Formats and Encoders

### Video Encoders

| Encoder | Format | Description |
|---------|--------|-------------|
| `x264enc` | H.264 | Widely supported, good quality |
| `x265enc` | H.265/HEVC | Better compression, newer |
| `vp8enc` | VP8 | WebM format |
| `vp9enc` | VP9 | WebM format, better quality |
| `theoraenc` | Theora | Ogg format |
| `av1enc` | AV1 | Modern, high efficiency |

### Audio Encoders

| Encoder | Format | Description |
|---------|--------|-------------|
| `lamemp3enc` | MP3 | Universal compatibility |
| `faac` | AAC | Standard for MP4 |
| `vorbisenc` | Vorbis | For WebM/Ogg |
| `flacenc` | FLAC | Lossless |
| `opusenc` | Opus | Modern, high quality |

### Container Formats (Muxers)

| Muxer | Extension | Description |
|-------|-----------|-------------|
| `mp4mux` | .mp4 | Most common format |
| `webmmux` | .webm | Open web format |
| `avimux` | .avi | Classic format |
| `oggmux` | .ogg | Open container |
| `matroskamux` | .mkv | Feature-rich |

---

## Examples

### 1. Simple Test Video

```typescript
import { GstKit } from '../index.js'

const kit = new GstKit()

kit.setPipeline(
  'videotestsrc pattern=snow num-buffers=300 ! ' +
  'video/x-raw,width=640,height=480,framerate=30/1 ! ' +
  'x264enc ! ' +
  'mp4mux ! ' +
  'filesink location=test.mp4'
)

kit.play()
// Wait for completion...
kit.stop()
kit.cleanup()
```

### 2. Video with Audio

```typescript
kit.setPipeline(
  'videotestsrc pattern=colors num-buffers=300 ! ' +
  'video/x-raw,width=1280,height=720,framerate=30/1 ! ' +
  'x264enc ! ' +
  'queue ! ' +
  'mp4mux name=mux ! ' +
  'filesink location=output.mp4 ' +
  'audiotestsrc wave=sine num-buffers=300 ! ' +
  'audio/x-raw,rate=44100,channels=2 ! ' +
  'lamemp3enc ! ' +
  'queue ! ' +
  'mux.'
)
```

### 3. Custom Video from Data

```typescript
kit.setPipeline(
  'appsrc name=source ! ' +
  'video/x-raw,width=320,height=240,format=RGB,framerate=30/1 ! ' +
  'videoconvert ! ' +
  'x264enc ! ' +
  'mp4mux ! ' +
  'filesink location=custom.mp4'
)

kit.play()

// Push custom frames
const frameSize = 320 * 240 * 3 // RGB
for (let i = 0; i < 300; i++) {
  const buffer = generateCustomFrame()
  kit.pushSample('source', buffer)
  await new Promise(resolve => setTimeout(resolve, 33))
}
```

### 4. Animated with Timestamp

```typescript
kit.setPipeline(
  'videotestsrc pattern=ball num-buffers=300 ! ' +
  'video/x-raw,width=800,height=600,framerate=30/1 ! ' +
  'timeoverlay font-desc="Sans, 48" ! ' +
  'x264enc ! ' +
  'mp4mux ! ' +
  'filesink location=animated.mp4'
)
```

---

## FFmpeg vs GStreamer

### Does FFmpeg have GStreamer integrated?

**Yes**, FFmpeg can use GStreamer as a backend for some operations, but they are separate tools:

#### FFmpeg with GStreamer:
```bash
# FFmpeg can use GStreamer as input
ffmpeg -f gstreamer -i "videotestsrc ! video/x-raw,width=640,height=480 ! fakesink" output.mp4
```

#### Key Differences:

| Feature | FFmpeg | GStreamer |
|---------|--------|-----------|
| **Architecture** | Command-line tool + libraries | Plugin-based framework |
| **Real-time** | Limited | Excellent |
| **Pipeline flexibility** | Moderate | Very high |
| **Integration** | Standalone | Can be embedded |
| **Live streaming** | Possible | Native support |
| **Custom processing** | Filters | Custom elements |

### When to use each:

**Use FFmpeg when:**
- Converting between formats
- Simple video processing
- Batch processing files
- Need wide codec support

**Use GStreamer when:**
- Real-time streaming
- Custom pipeline logic
- Live video processing
- Need to embed in applications
- Complex multi-element pipelines

### Interoperability:

You can actually use both together! For example:

```typescript
// Use GStreamer to capture and process
const kit = new GstKit()
kit.setPipeline(
  'videotestsrc ! ' +
  'video/x-raw,width=640,height=480 ! ' +
  'videoconvert ! ' +
  'appsink name=sink'
)

// Then feed to FFmpeg for encoding
// (This would require additional integration)
```

---

## Requirements

### System Requirements

To generate videos with GstKit, you need:

1. **GStreamer installed** with the following plugins:
   - `gstreamer1.0-plugins-base`
   - `gstreamer1.0-plugins-good` (test sources, encoders)
   - `gstreamer1.0-plugins-bad` (additional encoders)
   - `gstreamer1.0-plugins-ugly` (MP3 encoder)

2. **Installation commands:**

   **Ubuntu/Debian:**
   ```bash
   sudo apt-get install gstreamer1.0-tools \
     gstreamer1.0-plugins-base \
     gstreamer1.0-plugins-good \
     gstreamer1.0-plugins-bad \
     gstreamer1.0-plugins-ugly \
     gstreamer1.0-libav
   ```

   **Fedora:**
   ```bash
   sudo dnf install gstreamer1-plugins-base \
     gstreamer1-plugins-good \
     gstreamer1-plugins-bad \
     gstreamer1-plugins-bad-free \
     gstreamer1-plugins-ugly \
     gstreamer1-libav
   ```

   **macOS:**
   ```bash
   brew install gstreamer gst-libav gst-plugins-good \
     gst-plugins-bad gst-plugins-ugly
   ```

3. **Node.js environment** with GstKit installed

### Optional Requirements

For better performance and more codecs:
- Hardware acceleration plugins (VA-API, NVENC, etc.)
- Additional encoders (x265, AV1)
- Audio processing plugins

---

## Advanced Features

### 1. Multiple Video Streams

```typescript
kit.setPipeline(
  'videotestsrc pattern=colors ! ' +
  'video/x-raw,width=640,height=480 ! ' +
  'x264enc ! ' +
  'queue ! ' +
  'mp4mux name=mux ! ' +
  'filesink location=multi.mp4 ' +
  'videotestsrc pattern=snow ! ' +
  'video/x-raw,width=640,height=480 ! ' +
  'x264enc ! ' +
  'queue ! ' +
  'mux.'
)
```

### 2. Real-time Streaming

```typescript
kit.setPipeline(
  'videotestsrc ! ' +
  'video/x-raw,width=640,height=480,framerate=30/1 ! ' +
  'x264enc tune=zerolatency ! ' +
  'rtph264pay ! ' +
  'udpsink host=127.0.0.1 port=5000'
)
```

### 3. Video Effects

```typescript
kit.setPipeline(
  'videotestsrc ! ' +
  'video/x-raw,width=640,height=480 ! ' +
  'videobalance saturation=2.0 ! ' +
  'videoconvert ! ' +
  'x264enc ! ' +
  'mp4mux ! ' +
  'filesink location=effects.mp4'
)
```

### 4. Frame Rate Conversion

```typescript
kit.setPipeline(
  'videotestsrc ! ' +
  'video/x-raw,width=640,height=480,framerate=60/1 ! ' +
  'videorate ! ' +
  'video/x-raw,framerate=30/1 ! ' +
  'x264enc ! ' +
  'mp4mux ! ' +
  'filesink location=converted.mp4'
)
```

---

## Troubleshooting

### Common Issues

1. **"Failed to parse pipeline"**
   - Check GStreamer plugins are installed
   - Verify pipeline syntax
   - Use `gst-inspect-1.0` to check element availability

2. **"Failed to query position/duration"**
   - Ensure pipeline is in Playing or Paused state
   - Some formats don't support duration queries

3. **No output file created**
   - Check file permissions
   - Ensure muxer is properly connected
   - Wait for pipeline to finish before cleanup

4. **Poor performance**
   - Use `tune=zerolatency` for real-time
   - Reduce resolution or frame rate
   - Consider hardware acceleration

### Debugging

Enable GStreamer debug logs:

```bash
export GST_DEBUG=3
bun run examples/generate-video.ts
```

---

## Conclusion

GstKit provides powerful video generation capabilities through GStreamer's extensive plugin ecosystem. You can create test videos, process custom data, and generate content for testing without needing external video files.

For more complex scenarios, you can combine GStreamer with FFmpeg or use them independently based on your specific needs.
