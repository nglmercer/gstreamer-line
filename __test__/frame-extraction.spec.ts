/**
 * Frame Extraction and Screenshot Tests
 *
 * Tests for extracting frames and taking screenshots from videos
 * using the setup utilities.
 */

import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import setup from './setup.js';
import * as fs from 'node:fs';

describe('Frame Extraction and Screenshots', () => {
  let testVideoPath: string;

  beforeAll(async () => {
    setup.setupTestDirectories();
    // Create a test video for frame extraction
    testVideoPath = await setup.generateTestVideo('test_frames.avi', 'snow', {
      width: 320,
      height: 240,
      framerate: 30,
      numBuffers: 60, // 2 seconds
    });
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  describe('Single Frame Extraction', () => {
    it('should extract a single frame from video', async () => {
      const frame = await setup.extractFrame(testVideoPath, 0);

      expect(frame).not.toBeNull();
      expect(frame!.data.length).toBeGreaterThan(0);
      expect(frame!.width).toBe(320);
      expect(frame!.height).toBe(240);
      expect(frame!.format).toBe('RGBA');
    });

    it('should extract frame at specific timestamp', async () => {
      const frame = await setup.extractFrame(testVideoPath, 500); // 500ms

      expect(frame).not.toBeNull();
      expect(frame!.timestamp).toBe(500);
    });

    it('should validate extracted frame', async () => {
      const frame = await setup.extractFrame(testVideoPath, 0);

      expect(setup.validateFrame(frame!)).toBe(true);
    });
  });

  describe('Multiple Frame Extraction', () => {
    it('should extract multiple frames', async () => {
      const frames = await setup.extractFrames(testVideoPath, 5, 100);

      expect(frames.length).toBeGreaterThan(0);
      expect(frames.length).toBeLessThanOrEqual(5);

      for (const frame of frames) {
        expect(setup.validateFrame(frame)).toBe(true);
      }
    });

    it('should extract frames at regular intervals', async () => {
      const frames = await setup.extractFrames(testVideoPath, 10, 50);

      expect(frames.length).toBeGreaterThan(0);

      // Check timestamps are increasing
      for (let i = 1; i < frames.length; i++) {
        expect(frames[i].timestamp).toBeGreaterThanOrEqual(frames[i - 1].timestamp);
      }
    });
  });

  describe('All Frames Extraction', () => {
    it('should extract all frames from video', async () => {
      let frameCount = 0;
      const frames = await setup.extractAllFrames(testVideoPath, (frame, index) => {
        frameCount = index + 1;
        expect(setup.validateFrame(frame)).toBe(true);
      });

      expect(frames.length).toBeGreaterThan(0);
      expect(frameCount).toBeGreaterThan(0);
    });

    it('should stop when video ends', async () => {
      const frames = await setup.extractAllFrames(testVideoPath);

      // Should extract all available frames (60 frames for 2 seconds at 30fps)
      expect(frames.length).toBeGreaterThan(0);
      expect(frames.length).toBeLessThanOrEqual(60);
    });
  });

  describe('Frame Shots/Screenshots', () => {
    it('should take a single frame shot', async () => {
      const outputPath = setup.FRAMES_DIR + '/shot_0.rgba';
      const success = await setup.takeFrameShot(testVideoPath, outputPath, 0);

      expect(success).toBe(true);
      expect(fs.existsSync(outputPath)).toBe(true);
    });

    it('should take multiple frame shots', async () => {
      const timestamps = [0, 100, 200, 300, 400];
      const savedFiles = await setup.takeFrameShots(
        testVideoPath,
        setup.FRAMES_DIR,
        timestamps,
        'multi_shot'
      );

      expect(savedFiles.length).toBeGreaterThan(0);
      expect(savedFiles.length).toBeLessThanOrEqual(timestamps.length);

      for (const file of savedFiles) {
        expect(fs.existsSync(file)).toBe(true);
      }
    });

    it('should take interval frame shots', async () => {
      const savedFiles = await setup.takeIntervalFrameShots(
        testVideoPath,
        setup.FRAMES_DIR,
        200, // every 200ms
        1000, // for 1 second
        'interval_shot'
      );

      expect(savedFiles.length).toBeGreaterThan(0);
      expect(savedFiles.length).toBeLessThanOrEqual(5); // 1000ms / 200ms = 5 shots
    });

    it('should save frame as PPM format', async () => {
      const frame = await setup.extractFrame(testVideoPath, 0);
      const outputPath = setup.FRAMES_DIR + '/frame.ppm';

      setup.saveFrameAsPPM(frame!, outputPath);

      expect(fs.existsSync(outputPath)).toBe(true);
      const stats = fs.statSync(outputPath);
      expect(stats.size).toBeGreaterThan(0);
    });

    it('should save frame as raw data', async () => {
      const frame = await setup.extractFrame(testVideoPath, 0);
      const outputPath = setup.FRAMES_DIR + '/frame.raw';

      setup.saveFrameAsRaw(frame!, outputPath);

      expect(fs.existsSync(outputPath)).toBe(true);
      const stats = fs.statSync(outputPath);
      expect(stats.size).toBeGreaterThan(0);
    });
  });

  describe('Frame Comparison', () => {
    it('should compare two identical frames', async () => {
      const frame1 = await setup.extractFrame(testVideoPath, 0);
      const frame2 = await setup.extractFrame(testVideoPath, 0);

      expect(setup.compareFrames(frame1!, frame2!)).toBe(true);
    });

    it('should detect different frames', async () => {
      const frame1 = await setup.extractFrame(testVideoPath, 0);
      const frame2 = await setup.extractFrame(testVideoPath, 500);

      expect(setup.compareFrames(frame1!, frame2!)).toBe(false);
    });

    it('should calculate frame difference', async () => {
      const frame1 = await setup.extractFrame(testVideoPath, 0);
      const frame2 = await setup.extractFrame(testVideoPath, 500);

      const diff = setup.calculateFrameDifference(frame1!, frame2!);

      expect(diff).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Custom Frame Generators', () => {
    it('should generate gradient frames', async () => {
      const generator = setup.createGradientFrameGenerator(160, 120);
      const frame = generator(0);

      expect(frame.length).toBe(160 * 120 * 3); // RGB
    });

    it('should generate noise frames', async () => {
      const generator = setup.createNoiseFrameGenerator(160, 120);
      const frame = generator(0);

      expect(frame.length).toBe(160 * 120 * 3); // RGB
    });

    it('should generate solid color frames', async () => {
      const generator = setup.createSolidColorFrameGenerator(160, 120, 255, 0, 0);
      const frame = generator(0);

      expect(frame.length).toBe(160 * 120 * 3); // RGB
      expect(frame[0]).toBe(255); // Red
      expect(frame[1]).toBe(0);   // Green
      expect(frame[2]).toBe(0);   // Blue
    });

    it('should create video with custom gradient frames', async () => {
      const generator = setup.createGradientFrameGenerator(160, 120);
      const videoPath = await setup.generateCustomVideo(
        'gradient.avi',
        generator,
        { width: 160, height: 120, numBuffers: 30 }
      );

      expect(fs.existsSync(videoPath)).toBe(true);
    });

    it('should create video with custom noise frames', async () => {
      const generator = setup.createNoiseFrameGenerator(160, 120);
      const videoPath = await setup.generateCustomVideo(
        'noise.avi',
        generator,
        { width: 160, height: 120, numBuffers: 30 }
      );

      expect(fs.existsSync(videoPath)).toBe(true);
    });
  });

  describe('Video Generation with Different Patterns', () => {
    it('should generate video with red pattern', async () => {
      const videoPath = await setup.generateTestVideo('red.avi', 'red');

      expect(fs.existsSync(videoPath)).toBe(true);
    });

    it('should generate video with colors pattern', async () => {
      const videoPath = await setup.generateTestVideo('colors.avi', 'colors');

      expect(fs.existsSync(videoPath)).toBe(true);
    });

    it('should generate video with ball pattern', async () => {
      const videoPath = await setup.generateTestVideo('ball.avi', 'ball');

      expect(fs.existsSync(videoPath)).toBe(true);
    });
  });

  describe('Video Generation with Audio', () => {
    it('should generate video with sine wave audio', async () => {
      const videoPath = await setup.generateTestVideoWithAudio(
        'sine_audio.avi',
        'colors',
        'sine',
        { numBuffers: 30 }
      );

      expect(fs.existsSync(videoPath)).toBe(true);
    });

    it('should generate video with square wave audio', async () => {
      const videoPath = await setup.generateTestVideoWithAudio(
        'square_audio.avi',
        'colors',
        'square',
        { numBuffers: 30 }
      );

      expect(fs.existsSync(videoPath)).toBe(true);
    });
  });
});

describe('Benchmark Utilities', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should create benchmark video', async () => {
    const videoPath = await setup.createBenchmarkVideo('bench.avi', 2, {
      width: 320,
      height: 240,
      framerate: 30,
    });

    expect(fs.existsSync(videoPath)).toBe(true);
  });

  it('should benchmark frame extraction', async () => {
    const videoPath = await setup.createBenchmarkVideo('bench_extract.avi', 1, {
      width: 320,
      height: 240,
      framerate: 30,
    });

    const results = await setup.benchmarkFrameExtraction(videoPath, 3);

    expect(results.avgTime).toBeGreaterThan(0);
    expect(results.minTime).toBeGreaterThan(0);
    expect(results.maxTime).toBeGreaterThan(0);
    expect(results.maxTime).toBeGreaterThanOrEqual(results.minTime);
  });

  it('should benchmark frame shot', async () => {
    const videoPath = await setup.createBenchmarkVideo('bench_shot.avi', 1, {
      width: 320,
      height: 240,
      framerate: 30,
    });

    const results = await setup.benchmarkFrameShot(videoPath, 3);

    expect(results.avgTime).toBeGreaterThan(0);
    expect(results.minTime).toBeGreaterThan(0);
    expect(results.maxTime).toBeGreaterThan(0);
    expect(results.maxTime).toBeGreaterThanOrEqual(results.minTime);
  });
});
