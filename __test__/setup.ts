/**
 * Test Setup Utilities for GstKit
 *
 * This file provides reusable utilities for:
 * - Creating test videos
 * - Extracting frames from videos
 * - Taking frame shots/screenshots
 * - Common test setup/teardown
 * - Benchmark utilities
 */

import { GstKit } from '../index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';

// ============================================================================
// Configuration
// ============================================================================

export const TEST_DIR = path.join(__dirname, 'temp_output');
export const FRAMES_DIR = path.join(TEST_DIR, 'frames');

export interface VideoConfig {
  width: number;
  height: number;
  framerate: number;
  format: string;
  numBuffers: number;
  duration: number; // in seconds
}

export const DEFAULT_VIDEO_CONFIG: VideoConfig = {
  width: 320,
  height: 240,
  framerate: 30,
  format: 'RGB',
  numBuffers: 30,
  duration: 1,
};

export const VIDEO_PATTERNS = [
  'snow',
  'red',
  'green',
  'blue',
  'white',
  'black',
  'colors',
  'smpte',
  'ball',
  'checkers-1',
  'checkers-2',
  'checkers-4',
  'checkers-8',
  'circular',
  'gradient',
] as const;

export type VideoPattern = typeof VIDEO_PATTERNS[number];

// ============================================================================
// Directory Management
// ============================================================================

export function setupTestDirectories() {
  if (!fs.existsSync(TEST_DIR)) {
    fs.mkdirSync(TEST_DIR, { recursive: true });
  }
  if (!fs.existsSync(FRAMES_DIR)) {
    fs.mkdirSync(FRAMES_DIR, { recursive: true });
  }
}

export function cleanupTestDirectories() {
  if (fs.existsSync(FRAMES_DIR)) {
    const files = fs.readdirSync(FRAMES_DIR);
    for (const file of files) {
      fs.unlinkSync(path.join(FRAMES_DIR, file));
    }
    fs.rmdirSync(FRAMES_DIR);
  }

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
}

// ============================================================================
// Video Generation Utilities
// ============================================================================

/**
 * Generate a test video file
 */
export async function generateTestVideo(
  filename: string,
  pattern: VideoPattern = 'snow',
  config: Partial<VideoConfig> = {}
): Promise<string> {
  const finalConfig = { ...DEFAULT_VIDEO_CONFIG, ...config };
  const outputPath = path.join(TEST_DIR, filename);

  const kit = new GstKit();

  const pipeline = `
    videotestsrc pattern=${pattern} num-buffers=${finalConfig.numBuffers} !
    video/x-raw,width=${finalConfig.width},height=${finalConfig.height},framerate=${finalConfig.framerate}/1 !
    jpegenc ! avimux ! filesink location="${outputPath}"
  `;

  kit.setPipeline(pipeline);
  kit.play();

  // Wait for video generation to complete
  const waitTime = (finalConfig.numBuffers / finalConfig.framerate) * 1000 + 500;
  await new Promise(resolve => setTimeout(resolve, waitTime));

  kit.stop();
  kit.cleanup();

  return outputPath;
}

/**
 * Generate a test video with audio
 */
export async function generateTestVideoWithAudio(
  filename: string,
  videoPattern: VideoPattern = 'colors',
  audioWave: 'sine' | 'square' | 'saw' | 'triangle' = 'sine',
  config: Partial<VideoConfig> = {}
): Promise<string> {
  const finalConfig = { ...DEFAULT_VIDEO_CONFIG, ...config };
  const outputPath = path.join(TEST_DIR, filename);

  const kit = new GstKit();

  const pipeline = `
    videotestsrc pattern=${videoPattern} num-buffers=${finalConfig.numBuffers} !
    video/x-raw,width=${finalConfig.width},height=${finalConfig.height},framerate=${finalConfig.framerate}/1 !
    jpegenc ! queue ! avimux name=mux ! filesink location="${outputPath}"
    audiotestsrc wave=${audioWave} num-buffers=${finalConfig.numBuffers} !
    audio/x-raw,rate=44100,channels=2 ! lamemp3enc ! queue ! mux.
  `;

  kit.setPipeline(pipeline);
  kit.play();

  const waitTime = (finalConfig.numBuffers / finalConfig.framerate) * 1000 + 500;
  await new Promise(resolve => setTimeout(resolve, waitTime));

  kit.stop();
  kit.cleanup();

  return outputPath;
}

/**
 * Generate a test video from custom data
 */
export async function generateCustomVideo(
  filename: string,
  frameGenerator: (frameIndex: number) => Buffer,
  config: Partial<VideoConfig> = {}
): Promise<string> {
  const finalConfig = { ...DEFAULT_VIDEO_CONFIG, ...config };
  const outputPath = path.join(TEST_DIR, filename);

  const kit = new GstKit();

  const pipeline = `
    appsrc name=source !
    video/x-raw,width=${finalConfig.width},height=${finalConfig.height},format=${finalConfig.format},framerate=${finalConfig.framerate}/1 !
    videoconvert ! jpegenc ! avimux ! filesink location="${outputPath}"
  `;

  kit.setPipeline(pipeline);
  kit.play();

  // Generate and push frames
  for (let i = 0; i < finalConfig.numBuffers; i++) {
    const frame = frameGenerator(i);
    kit.pushSample('source', frame);
    // Throttle to target framerate
    await new Promise(resolve => setTimeout(resolve, 1000 / finalConfig.framerate));
  }

  kit.stop();
  kit.cleanup();

  return outputPath;
}

// ============================================================================
// Frame Extraction Utilities
// ============================================================================

export interface Frame {
  data: Buffer;
  width: number;
  height: number;
  format: string;
  timestamp: number;
}

/**
 * Extract a single frame from a video
 */
export async function extractFrame(
  videoPath: string,
  timestampMs: number = 0
): Promise<Frame | null> {
  const kit = new GstKit();

  const pipeline = `
    filesrc location="${videoPath}" !
    avidemux ! jpegdec !
    videoconvert ! video/x-raw,format=RGBA !
    appsink name=sink
  `;

  kit.setPipeline(pipeline);
  kit.play();

  // Wait for pre-roll
  await new Promise(resolve => setTimeout(resolve, 500));

  // Seek to timestamp if needed
  if (timestampMs > 0) {
    kit.seek(timestampMs * 1_000_000); // Convert to nanoseconds
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  // Pull frame
  const frameData = kit.pullSample('sink');
  kit.stop();
  kit.cleanup();

  if (!frameData) {
    return null;
  }

  return {
    data: frameData,
    width: 320, // Default, can be detected
    height: 240,
    format: 'RGBA',
    timestamp: timestampMs,
  };
}

/**
 * Extract multiple frames from a video
 */
export async function extractFrames(
  videoPath: string,
  count: number = 10,
  intervalMs: number = 100
): Promise<Frame[]> {
  const kit = new GstKit();

  const pipeline = `
    filesrc location="${videoPath}" !
    avidemux ! jpegdec !
    videoconvert ! video/x-raw,format=RGBA !
    appsink name=sink
  `;

  kit.setPipeline(pipeline);
  kit.play();

  await new Promise(resolve => setTimeout(resolve, 500));

  const frames: Frame[] = [];
  let lastTimestamp = 0;

  for (let i = 0; i < count; i++) {
    const frameData = kit.pullSample('sink');
    if (frameData) {
      frames.push({
        data: frameData,
        width: 320,
        height: 240,
        format: 'RGBA',
        timestamp: lastTimestamp,
      });
    }

    lastTimestamp += intervalMs;
    if (i < count - 1) {
      await new Promise(resolve => setTimeout(resolve, intervalMs));
    }
  }

  kit.stop();
  kit.cleanup();

  return frames;
}

/**
 * Extract all frames from a video
 */
export async function extractAllFrames(
  videoPath: string,
  onFrame?: (frame: Frame, index: number) => void
): Promise<Frame[]> {
  const kit = new GstKit();

  const pipeline = `
    filesrc location="${videoPath}" !
    avidemux ! jpegdec !
    videoconvert ! video/x-raw,format=RGBA !
    appsink name=sink
  `;

  kit.setPipeline(pipeline);
  kit.play();

  await new Promise(resolve => setTimeout(resolve, 500));

  const frames: Frame[] = [];
  let index = 0;
  let consecutiveNullFrames = 0;
  const maxNullFrames = 10;

  while (consecutiveNullFrames < maxNullFrames) {
    const frameData = kit.pullSample('sink');
    if (frameData) {
      const frame = {
        data: frameData,
        width: 320,
        height: 240,
        format: 'RGBA',
        timestamp: index * (1000 / 30), // Assume 30fps
      };
      frames.push(frame);
      if (onFrame) {
        onFrame(frame, index);
      }
      consecutiveNullFrames = 0;
    } else {
      consecutiveNullFrames++;
    }
    index++;
    await new Promise(resolve => setTimeout(resolve, 33)); // ~30fps
  }

  kit.stop();
  kit.cleanup();

  return frames;
}

// ============================================================================
// Frame Shot/Screenshot Utilities
// ============================================================================

/**
 * Take a frame shot at a specific timestamp
 */
export async function takeFrameShot(
  videoPath: string,
  outputPath: string,
  timestampMs: number = 0
): Promise<boolean> {
  const frame = await extractFrame(videoPath, timestampMs);

  if (!frame) {
    return false;
  }

  // Save frame as raw RGBA data
  fs.writeFileSync(outputPath, frame.data);

  return true;
}

/**
 * Take multiple frame shots at different timestamps
 */
export async function takeFrameShots(
  videoPath: string,
  outputDir: string,
  timestampsMs: number[],
  prefix: string = 'frame'
): Promise<string[]> {
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const savedFiles: string[] = [];

  for (let i = 0; i < timestampsMs.length; i++) {
    const outputPath = path.join(outputDir, `${prefix}_${i.toString().padStart(3, '0')}.rgba`);
    const success = await takeFrameShot(videoPath, outputPath, timestampsMs[i]);
    if (success) {
      savedFiles.push(outputPath);
    }
  }

  return savedFiles;
}

/**
 * Take frame shots at regular intervals
 */
export async function takeIntervalFrameShots(
  videoPath: string,
  outputDir: string,
  intervalMs: number = 100,
  durationMs: number = 1000,
  prefix: string = 'frame'
): Promise<string[]> {
  const timestamps: number[] = [];
  for (let t = 0; t < durationMs; t += intervalMs) {
    timestamps.push(t);
  }

  return takeFrameShots(videoPath, outputDir, timestamps, prefix);
}

/**
 * Save frame as PPM format (simple image format)
 */
export function saveFrameAsPPM(frame: Frame, outputPath: string): void {
  const header = `P6\n${frame.width} ${frame.height}\n255\n`;
  const buffer = Buffer.concat([
    Buffer.from(header),
    frame.data,
  ]);
  fs.writeFileSync(outputPath, buffer);
}

/**
 * Save frame as raw binary data
 */
export function saveFrameAsRaw(frame: Frame, outputPath: string): void {
  fs.writeFileSync(outputPath, frame.data);
}

// ============================================================================
// Benchmark Utilities
// ============================================================================

/**
 * Create a benchmark video for performance testing
 */
export async function createBenchmarkVideo(
  filename: string = 'benchmark.avi',
  duration: number = 10,
  config: Partial<VideoConfig> = {}
): Promise<string> {
  const baseConfig = { ...DEFAULT_VIDEO_CONFIG, ...config };
  const finalConfig = {
    ...baseConfig,
    numBuffers: duration * baseConfig.framerate,
  };

  return generateTestVideo(filename, 'snow', finalConfig);
}

/**
 * Benchmark frame extraction performance
 */
export async function benchmarkFrameExtraction(
  videoPath: string,
  iterations: number = 100
): Promise<{ avgTime: number; minTime: number; maxTime: number }> {
  const times: number[] = [];

  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await extractFrame(videoPath, 0);
    const end = performance.now();
    times.push(end - start);
  }

  const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
  const minTime = Math.min(...times);
  const maxTime = Math.max(...times);

  return { avgTime, minTime, maxTime };
}

/**
 * Benchmark frame shot performance
 */
export async function benchmarkFrameShot(
  videoPath: string,
  iterations: number = 100
): Promise<{ avgTime: number; minTime: number; maxTime: number }> {
  const times: number[] = [];

  for (let i = 0; i < iterations; i++) {
    const outputPath = path.join(FRAMES_DIR, `bench_${i}.rgba`);
    const start = performance.now();
    await takeFrameShot(videoPath, outputPath, 0);
    const end = performance.now();
    times.push(end - start);

    // Cleanup
    if (fs.existsSync(outputPath)) {
      fs.unlinkSync(outputPath);
    }
  }

  const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
  const minTime = Math.min(...times);
  const maxTime = Math.max(...times);

  return { avgTime, minTime, maxTime };
}

// ============================================================================
// Test Helpers
// ============================================================================

/**
 * Create a simple gradient frame generator
 */
export function createGradientFrameGenerator(
  width: number,
  height: number
): (frameIndex: number) => Buffer {
  const frameSize = width * height * 3; // RGB

  return (frameIndex: number) => {
    const buffer = Buffer.alloc(frameSize);
    const offset = frameIndex * 2;

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const pixelOffset = (y * width + x) * 3;
        const r = Math.floor(((x + offset) % width) / width * 255);
        const g = Math.floor(((y + offset) % height) / height * 255);
        const b = Math.floor(((x + y + offset) % (width + height)) / (width + height) * 255);
        buffer[pixelOffset] = r;
        buffer[pixelOffset + 1] = g;
        buffer[pixelOffset + 2] = b;
      }
    }

    return buffer;
  };
}

/**
 * Create a noise frame generator
 */
export function createNoiseFrameGenerator(
  width: number,
  height: number
): (frameIndex: number) => Buffer {
  const frameSize = width * height * 3;

  return () => {
    const buffer = Buffer.alloc(frameSize);
    for (let i = 0; i < frameSize; i++) {
      buffer[i] = Math.floor(Math.random() * 256);
    }
    return buffer;
  };
}

/**
 * Create a solid color frame generator
 */
export function createSolidColorFrameGenerator(
  width: number,
  height: number,
  r: number,
  g: number,
  b: number
): (frameIndex: number) => Buffer {
  const frameSize = width * height * 3;
  const color = Buffer.from([r, g, b]);

  return () => {
    const buffer = Buffer.alloc(frameSize);
    for (let i = 0; i < frameSize; i += 3) {
      buffer[i] = color[0];
      buffer[i + 1] = color[1];
      buffer[i + 2] = color[2];
    }
    return buffer;
  };
}

// ============================================================================
// Validation Utilities
// ============================================================================

/**
 * Validate a frame
 */
export function validateFrame(frame: Frame): boolean {
  if (!frame || !frame.data || frame.data.length === 0) {
    return false;
  }

  const expectedSize = frame.width * frame.height * 4; // RGBA
  return frame.data.length === expectedSize;
}

/**
 * Compare two frames for equality
 */
export function compareFrames(frame1: Frame, frame2: Frame): boolean {
  if (frame1.width !== frame2.width || frame1.height !== frame2.height) {
    return false;
  }

  if (frame1.data.length !== frame2.data.length) {
    return false;
  }

  for (let i = 0; i < frame1.data.length; i++) {
    if (frame1.data[i] !== frame2.data[i]) {
      return false;
    }
  }

  return true;
}

/**
 * Calculate frame difference (for testing)
 */
export function calculateFrameDifference(frame1: Frame, frame2: Frame): number {
  if (frame1.width !== frame2.width || frame1.height !== frame2.height) {
    return -1;
  }

  let diff = 0;
  for (let i = 0; i < frame1.data.length; i++) {
    diff += Math.abs(frame1.data[i] - frame2.data[i]);
  }

  return diff;
}

// ============================================================================
// Export All
// ============================================================================

export default {
  // Directory management
  setupTestDirectories,
  cleanupTestDirectories,

  // Video generation
  generateTestVideo,
  generateTestVideoWithAudio,
  generateCustomVideo,

  // Frame extraction
  extractFrame,
  extractFrames,
  extractAllFrames,

  // Frame shots
  takeFrameShot,
  takeFrameShots,
  takeIntervalFrameShots,
  saveFrameAsPPM,
  saveFrameAsRaw,

  // Benchmark utilities
  createBenchmarkVideo,
  benchmarkFrameExtraction,
  benchmarkFrameShot,

  // Test helpers
  createGradientFrameGenerator,
  createNoiseFrameGenerator,
  createSolidColorFrameGenerator,

  // Validation
  validateFrame,
  compareFrames,
  calculateFrameDifference,

  // Configuration
  TEST_DIR,
  FRAMES_DIR,
  DEFAULT_VIDEO_CONFIG,
  VIDEO_PATTERNS,
};
