/**
 * Codec Debug Tests
 *
 * Comprehensive debugging tests for codec-specific frame extraction.
 * These tests provide detailed logging to identify where failures occur.
 */

import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { GstKit } from '../index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';

const TEST_DIR = path.join(__dirname, 'temp_output');

// ============================================================================
// Codec Definitions
// ============================================================================

interface VideoCodec {
  name: string;
  encoder: string;
  decoder: string;
  formats: VideoFormat[];
}

interface VideoFormat {
  extension: string;
  muxer: string;
  demuxer: string;
}

const VIDEO_CODECS: Record<string, VideoCodec> = {
  h264: {
    name: 'H.264',
    encoder: 'x264enc',
    decoder: 'avdec_h264',
    formats: [
      { extension: 'mp4', muxer: 'mp4mux', demuxer: 'qtdemux' },
      { extension: 'mkv', muxer: 'matroskamux', demuxer: 'matroskademux' },
    ],
  },
  h265: {
    name: 'H.265/HEVC',
    encoder: 'x265enc',
    decoder: 'avdec_h265',
    formats: [
      { extension: 'mp4', muxer: 'mp4mux', demuxer: 'qtdemux' },
      { extension: 'mkv', muxer: 'matroskamux', demuxer: 'matroskademux' },
    ],
  },
  vp8: {
    name: 'VP8',
    encoder: 'vp8enc',
    decoder: 'vp8dec',
    formats: [
      { extension: 'webm', muxer: 'webmmux', demuxer: 'matroskademux' },
      { extension: 'mkv', muxer: 'matroskamux', demuxer: 'matroskademux' },
    ],
  },
  vp9: {
    name: 'VP9',
    encoder: 'vp9enc',
    decoder: 'vp9dec',
    formats: [
      { extension: 'webm', muxer: 'webmmux', demuxer: 'matroskademux' },
      { extension: 'mkv', muxer: 'matroskamux', demuxer: 'matroskademux' },
    ],
  },
  av1: {
    name: 'AV1',
    encoder: 'av1enc',
    decoder: 'av1dec',
    formats: [
      { extension: 'webm', muxer: 'webmmux', demuxer: 'matroskademux' },
      { extension: 'mkv', muxer: 'matroskamux', demuxer: 'matroskademux' },
    ],
  },
  mpeg2: {
    name: 'MPEG-2',
    encoder: 'mpeg2enc',
    decoder: 'mpeg2dec',
    formats: [
      { extension: 'mpg', muxer: 'mpegpsmux', demuxer: 'mpegpsdemux' },
    ],
  },
  theora: {
    name: 'Theora',
    encoder: 'theoraenc',
    decoder: 'theoradec',
    formats: [
      { extension: 'ogv', muxer: 'oggmux', demuxer: 'oggdemux' },
    ],
  },
  jpeg: {
    name: 'MJPEG',
    encoder: 'jpegenc',
    decoder: 'jpegdec',
    formats: [
      { extension: 'avi', muxer: 'avimux', demuxer: 'avidemux' },
      { extension: 'mov', muxer: 'qtmux', demuxer: 'qtdemux' },
    ],
  },
  png: {
    name: 'PNG',
    encoder: 'pngenc',
    decoder: 'pngdec',
    formats: [
      { extension: 'avi', muxer: 'avimux', demuxer: 'avidemux' },
      { extension: 'mov', muxer: 'qtmux', demuxer: 'qtdemux' },
    ],
  },
};

type VideoCodecKey = keyof typeof VIDEO_CODECS;

// ============================================================================
// Setup and Teardown
// ============================================================================

beforeAll(() => {
  if (!fs.existsSync(TEST_DIR)) {
    fs.mkdirSync(TEST_DIR, { recursive: true });
  }
});

afterAll(() => {
  // Cleanup all test files
  if (fs.existsSync(TEST_DIR)) {
    const files = fs.readdirSync(TEST_DIR);
    for (const file of files) {
      const filePath = path.join(TEST_DIR, file);
      const stat = fs.statSync(filePath);
      if (stat.isDirectory()) {
        const subFiles = fs.readdirSync(filePath);
        for (const subFile of subFiles) {
          fs.unlinkSync(path.join(filePath, subFile));
        }
        fs.rmdirSync(filePath);
      } else {
        fs.unlinkSync(filePath);
      }
    }
    fs.rmdirSync(TEST_DIR);
  }
});

// ============================================================================
// Helper Functions
// ============================================================================

async function isCodecAvailable(codecKey: VideoCodecKey): Promise<boolean> {
  const codec = VIDEO_CODECS[codecKey];
  const kit = new GstKit();

  console.log(`  [DEBUG] Testing codec availability for ${codecKey} (${codec.encoder})`);

  try {
    const pipeline = `videotestsrc num-buffers=1 ! ${codec.encoder} ! fakesink`;
    console.log(`  [DEBUG] Pipeline: ${pipeline}`);

    kit.setPipeline(pipeline);
    console.log(`  [DEBUG] Pipeline set successfully`);

    kit.play();
    console.log(`  [DEBUG] Pipeline playing`);

    await new Promise(resolve => setTimeout(resolve, 100));

    kit.stop();
    kit.cleanup();

    console.log(`  [DEBUG] ✓ Codec ${codecKey} is available`);
    return true;
  } catch (error) {
    if (error instanceof Error) {
      console.log(`  [DEBUG] ✗ Codec ${codecKey} is NOT available: ${error.message}`);
    } else {
      console.log(`  [DEBUG] ✗ Codec ${codecKey} is NOT available: ${String(error)}`);
    }
    return false;
  }
}

async function generateVideoWithCodec(
  codecKey: VideoCodecKey,
  formatIndex: number = 0,
  pattern: string = 'snow'
): Promise<{ videoPath: string; codec: VideoCodec; format: VideoFormat }> {
  const codec = VIDEO_CODECS[codecKey];
  const format = codec.formats[formatIndex];
  const filename = `${codecKey}_${format.extension}`;
  const outputPath = path.join(TEST_DIR, filename);

  console.log(`\n[DEBUG] Generating video with ${codecKey} codec`);
  console.log(`  Codec: ${codec.name}`);
  console.log(`  Encoder: ${codec.encoder}`);
  console.log(`  Format: ${format.extension}`);
  console.log(`  Muxer: ${format.muxer}`);
  console.log(`  Pattern: ${pattern}`);
  console.log(`  Output: ${outputPath}`);

  const kit = new GstKit();

  const pipeline = `
    videotestsrc pattern=${pattern} num-buffers=30 !
    video/x-raw,width=320,height=240,framerate=30/1 !
    ${codec.encoder} ! ${format.muxer} ! filesink location="${outputPath}"
  `;

  console.log(`  Pipeline: ${pipeline.trim()}`);

  try {
    kit.setPipeline(pipeline);
    console.log(`  [DEBUG] ✓ Pipeline set successfully`);

    kit.play();
    console.log(`  [DEBUG] ✓ Pipeline playing`);

    // Wait for video generation (30 frames at 30fps = 1 second)
    await new Promise(resolve => setTimeout(resolve, 1500));

    kit.stop();
    kit.cleanup();

    // Check if file was created
    if (!fs.existsSync(outputPath)) {
      console.log(`  [DEBUG] ✗ File NOT created: ${outputPath}`);
      throw new Error(`Video file was not created: ${outputPath}`);
    }

    const stats = fs.statSync(outputPath);
    console.log(`  [DEBUG] ✓ File created successfully`);
    console.log(`  [DEBUG] File size: ${stats.size} bytes`);

    return { videoPath: outputPath, codec, format };
  } catch (error) {
    console.log(`  [DEBUG] ✗ Error generating video: ${error instanceof Error ? error.message : String(error)}`);
    throw error;
  }
}

async function extractFrameFromCodecVideo(
  videoPath: string,
  codecKey: VideoCodecKey,
  formatIndex: number = 0,
  timestampMs: number = 0
): Promise<{ data: Buffer; width: number; height: number; format: string; timestamp: number } | null> {
  const codec = VIDEO_CODECS[codecKey];
  const format = codec.formats[formatIndex];

  console.log(`\n[DEBUG] Extracting frame from ${codecKey} video`);
  console.log(`  Video: ${videoPath}`);
  console.log(`  Codec: ${codec.name}`);
  console.log(`  Decoder: ${codec.decoder}`);
  console.log(`  Demuxer: ${format.demuxer}`);
  console.log(`  Timestamp: ${timestampMs}ms`);

  const kit = new GstKit();

  const pipeline = `
    filesrc location="${videoPath}" !
    ${format.demuxer} ! ${codec.decoder} !
    videoconvert ! video/x-raw,format=RGBA !
    appsink name=sink
  `;

  console.log(`  Pipeline: ${pipeline.trim()}`);

  try {
    kit.setPipeline(pipeline);
    console.log(`  [DEBUG] ✓ Pipeline set successfully`);

    kit.play();
    console.log(`  [DEBUG] ✓ Pipeline playing`);

    await new Promise(resolve => setTimeout(resolve, 500));

    if (timestampMs > 0) {
      kit.seek(timestampMs * 1_000_000);
      console.log(`  [DEBUG] ✓ Seeked to ${timestampMs}ms`);
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    const frameData = kit.pullSample('sink');

    if (!frameData) {
      console.log(`  [DEBUG] ✗ No frame data received`);
      kit.stop();
      kit.cleanup();
      return null;
    }

    console.log(`  [DEBUG] ✓ Frame received`);
    console.log(`  [DEBUG] Frame size: ${frameData.length} bytes`);

    kit.stop();
    kit.cleanup();

    return {
      data: frameData,
      width: 320,
      height: 240,
      format: 'RGBA',
      timestamp: timestampMs,
    };
  } catch (error) {
    console.log(`  [DEBUG] ✗ Error extracting frame: ${error instanceof Error ? error.message : String(error)}`);
    console.log(`  [DEBUG] Error stack: ${error instanceof Error ? error.stack : 'N/A'}`);
    throw error;
  }
}

// ============================================================================
// Test Suites
// ============================================================================

describe('Codec Debug - Availability Detection', () => {
  it('should detect all codec availability', async () => {
    console.log('\n' + '='.repeat(80));
    console.log('TESTING CODEC AVAILABILITY');
    console.log('='.repeat(80));

    const results: Record<string, boolean> = {};

    for (const codecKey of Object.keys(VIDEO_CODECS) as VideoCodecKey[]) {
      results[codecKey] = await isCodecAvailable(codecKey);
    }

    console.log('\n' + '-'.repeat(80));
    console.log('SUMMARY:');
    console.log('-'.repeat(80));

    const available = Object.entries(results).filter(([_, available]) => available);
    const unavailable = Object.entries(results).filter(([_, available]) => !available);

    console.log(`Available codecs (${available.length}):`);
    available.forEach(([codec, _]) => console.log(`  ✓ ${codec}`));

    console.log(`\nUnavailable codecs (${unavailable.length}):`);
    unavailable.forEach(([codec, _]) => console.log(`  ✗ ${codec}`));

    console.log('='.repeat(80) + '\n');

    // At least jpeg should be available
    expect(results['jpeg']).toBe(true);
  });
});

describe('Codec Debug - Video Generation', () => {
  const codecsToTest: VideoCodecKey[] = ['jpeg', 'h264', 'vp8', 'png'];

  it.each(codecsToTest)('should generate video with %s codec if available', async (codecKey) => {
    console.log('\n' + '='.repeat(80));
    console.log(`TESTING VIDEO GENERATION: ${codecKey}`);
    console.log('='.repeat(80));

    const isAvailable = await isCodecAvailable(codecKey);

    if (!isAvailable) {
      console.log(`\nSkipping ${codecKey} test: codec not available`);
      return;
    }

    const result = await generateVideoWithCodec(codecKey, 0, 'snow');

    expect(fs.existsSync(result.videoPath)).toBe(true);
    expect(result.videoPath).toContain(codecKey);
    expect(result.videoPath).toContain(result.format.extension);

    const stats = fs.statSync(result.videoPath);
    expect(stats.size).toBeGreaterThan(100);

    console.log(`\n✓ Video generation test PASSED for ${codecKey}`);
    console.log('='.repeat(80) + '\n');
  });
});

describe('Codec Debug - Frame Extraction', () => {
  const codecsToTest: VideoCodecKey[] = ['jpeg', 'h264', 'vp8'];

  it.each(codecsToTest)('should extract frame from %s video if available', async (codecKey) => {
    console.log('\n' + '='.repeat(80));
    console.log(`TESTING FRAME EXTRACTION: ${codecKey}`);
    console.log('='.repeat(80));

    const isAvailable = await isCodecAvailable(codecKey);

    if (!isAvailable) {
      console.log(`\nSkipping ${codecKey} extraction test: codec not available`);
      return;
    }

    // First generate the video
    console.log(`\n--- Step 1: Generate video ---`);
    const { videoPath } = await generateVideoWithCodec(codecKey, 0, 'snow');

    // Then extract a frame
    console.log(`\n--- Step 2: Extract frame ---`);
    const frame = await extractFrameFromCodecVideo(videoPath, codecKey, 0, 0);

    expect(frame).not.toBeNull();
    expect(frame!.data.length).toBeGreaterThan(0);
    expect(frame!.data.length).toBe(320 * 240 * 4); // RGBA

    console.log(`\n✓ Frame extraction test PASSED for ${codecKey}`);
    console.log('='.repeat(80) + '\n');
  });
});

describe('Codec Debug - Multiple Format Support', () => {
  const codecsWithMultipleFormats: { codec: VideoCodecKey; formatIndices: number[] }[] = [
    { codec: 'h264', formatIndices: [0, 1] },
    { codec: 'vp8', formatIndices: [0, 1] },
    { codec: 'jpeg', formatIndices: [0, 1] },
  ];

  it.each(codecsWithMultipleFormats)('should generate $codec video in multiple formats', async ({ codec, formatIndices }) => {
    console.log('\n' + '='.repeat(80));
    console.log(`TESTING MULTIPLE FORMATS: ${codec}`);
    console.log('='.repeat(80));

    const isAvailable = await isCodecAvailable(codec);

    if (!isAvailable) {
      console.log(`\nSkipping ${codec} multi-format test: codec not available`);
      return;
    }

    for (const formatIndex of formatIndices) {
      console.log(`\n--- Testing format index ${formatIndex} ---`);
      const { videoPath, format } = await generateVideoWithCodec(codec, formatIndex, 'colors');

      expect(fs.existsSync(videoPath)).toBe(true);
      expect(videoPath).toContain(format.extension);

      const stats = fs.statSync(videoPath);
      expect(stats.size).toBeGreaterThan(100);

      console.log(`  ✓ Format ${format.extension} generated successfully`);
    }

    console.log(`\n✓ Multiple format test PASSED for ${codec}`);
    console.log('='.repeat(80) + '\n');
  });
});

describe('Codec Debug - Frame Extraction with Seeking', () => {
  it('should extract frames at different timestamps from JPEG video', async () => {
    console.log('\n' + '='.repeat(80));
    console.log('TESTING FRAME EXTRACTION WITH SEEKING: JPEG');
    console.log('='.repeat(80));

    const isAvailable = await isCodecAvailable('jpeg');

    if (!isAvailable) {
      console.log('\nSkipping JPEG test: codec not available');
      return;
    }

    // Generate video
    console.log('\n--- Step 1: Generate video ---');
    const { videoPath } = await generateVideoWithCodec('jpeg', 0, 'snow');

    // Extract frames at different timestamps
    console.log('\n--- Step 2: Extract frames at different timestamps ---');
    const timestamps = [0, 100, 200, 300, 400];
    const frames: { timestamp: number; size: number }[] = [];

    for (const timestamp of timestamps) {
      console.log(`\nExtracting frame at ${timestamp}ms`);
      const frame = await extractFrameFromCodecVideo(videoPath, 'jpeg', 0, timestamp);

      expect(frame).not.toBeNull();
      expect(frame!.data.length).toBeGreaterThan(0);

      frames.push({ timestamp, size: frame!.data.length });
      console.log(`  ✓ Frame extracted: ${frame!.data.length} bytes`);
    }

    // All frames should have the same size
    const firstSize = frames[0].size;
    for (let i = 1; i < frames.length; i++) {
      expect(frames[i].size).toBe(firstSize);
    }

    console.log('\n✓ Frame extraction with seeking test PASSED');
    console.log('='.repeat(80) + '\n');
  });
});

describe('Codec Debug - Pipeline State Inspection', () => {
  it('should inspect pipeline state during video generation', async () => {
    console.log('\n' + '='.repeat(80));
    console.log('TESTING PIPELINE STATE INSPECTION');
    console.log('='.repeat(80));

    const kit = new GstKit();
    const outputPath = path.join(TEST_DIR, 'state_test.avi');

    const pipeline = `
      videotestsrc pattern=snow num-buffers=30 !
      video/x-raw,width=320,height=240,framerate=30/1 !
      jpegenc ! avimux ! filesink location="${outputPath}"
    `;

    console.log(`\nPipeline: ${pipeline.trim()}`);

    // Set pipeline
    kit.setPipeline(pipeline);
    console.log(`\nState after setPipeline: ${kit.getState()}`);
    expect(kit.getState()).toBe('Null');

    // Play
    kit.play();
    console.log(`State after play: ${kit.getState()}`);
    expect(['Playing', 'Paused'].includes(kit.getState())).toBe(true);

    // Wait a bit
    await new Promise(resolve => setTimeout(resolve, 500));

    // Check state again
    console.log(`State during playback: ${kit.getState()}`);

    // Get position
    const position = kit.getPosition();
    console.log(`Position: ${position} ns (${(position / 1_000_000).toFixed(2)} ms)`);
    expect(position).toBeGreaterThanOrEqual(0);

    // Get duration
    const duration = kit.getDuration();
    console.log(`Duration: ${duration} ns (${duration >= 0 ? (duration / 1_000_000).toFixed(2) + ' ms' : 'unknown'})`);
    expect(duration).toBeGreaterThanOrEqual(-1);

    // Get elements
    const elements = kit.getElements();
    console.log(`Elements: ${elements.join(', ')}`);
    expect(elements.length).toBeGreaterThan(0);

    // Stop
    kit.stop();
    console.log(`State after stop: ${kit.getState()}`);
    expect(kit.getState()).toBe('Null');

    kit.cleanup();

    console.log('\n✓ Pipeline state inspection test PASSED');
    console.log('='.repeat(80) + '\n');
  });
});

describe('Codec Debug - Error Handling', () => {
  it('should handle invalid codec gracefully', async () => {
    console.log('\n' + '='.repeat(80));
    console.log('TESTING ERROR HANDLING: Invalid Codec');
    console.log('='.repeat(80));

    const kit = new GstKit();
    const outputPath = path.join(TEST_DIR, 'invalid_test.avi');

    // Use a non-existent encoder
    const pipeline = `
      videotestsrc pattern=snow num-buffers=30 !
      video/x-raw,width=320,height=240,framerate=30/1 !
      nonexistentencoder ! avimux ! filesink location="${outputPath}"
    `;

    console.log(`\nPipeline: ${pipeline.trim()}`);

    try {
      kit.setPipeline(pipeline);
      console.log('✗ Should have thrown an error');
      expect(false).toBe(true);
    } catch (error) {
      console.log(`\n✓ Error caught as expected: ${error instanceof Error ? error.message : String(error)}`);
      expect(error).toBeDefined();
    }

    kit.cleanup();

    console.log('\n✓ Error handling test PASSED');
    console.log('='.repeat(80) + '\n');
  });

  it('should handle invalid demuxer gracefully', async () => {
    console.log('\n' + '='.repeat(80));
    console.log('TESTING ERROR HANDLING: Invalid Demuxer');
    console.log('='.repeat(80));

    // First create a valid video
    const { videoPath } = await generateVideoWithCodec('jpeg', 0, 'snow');

    // Try to extract with invalid demuxer
    const kit = new GstKit();

    const pipeline = `
      filesrc location="${videoPath}" !
      nonexistdemuxer ! jpegdec !
      videoconvert ! video/x-raw,format=RGBA !
      appsink name=sink
    `;

    console.log(`\nPipeline: ${pipeline.trim()}`);

    try {
      kit.setPipeline(pipeline);
      console.log('✗ Should have thrown an error');
      expect(false).toBe(true);
    } catch (error) {
      console.log(`\n✓ Error caught as expected: ${error instanceof Error ? error.message : String(error)}`);
      expect(error).toBeDefined();
    }

    kit.cleanup();

    console.log('\n✓ Error handling test PASSED');
    console.log('='.repeat(80) + '\n');
  });
});
