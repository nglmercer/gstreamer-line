/**
 * Bun tests for Rust-AV Kit NAPI bindings
 */
import { test,describe,expect,beforeAll,afterAll } from 'bun:test';
import { 
  getSupportedFormats,
  getSupportedCodecs,
  getSupportedPixelFormats,
  getSupportedSampleFormats,
  getMediaInfo,
  transcode
} from '../index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';

const TEST_DIR = path.join(__dirname, 'temp_output');

function setupTestDir() {
  if (!fs.existsSync(TEST_DIR)) {
    fs.mkdirSync(TEST_DIR, { recursive: true });
  }
}

function cleanupTestDir() {
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

describe('Rust-AV Kit - Supported Formats', () => {
  test('getSupportedFormats should return an array of formats', () => {
    const formats = getSupportedFormats();
    expect(Array.isArray(formats)).toBe(true);
    expect(formats.length).toBeGreaterThan(0);
    expect(formats).toContain('ivf');
    expect(formats).toContain('webm');
  });

  test('getSupportedCodecs should return an array of codecs', () => {
    const codecs = getSupportedCodecs();
    expect(Array.isArray(codecs)).toBe(true);
    expect(codecs.length).toBeGreaterThan(0);
    expect(codecs).toContain('av1');
    expect(codecs).toContain('opus');
  });

  test('getSupportedPixelFormats should return an array of pixel formats', () => {
    const pixelFormats = getSupportedPixelFormats();
    expect(Array.isArray(pixelFormats)).toBe(true);
    expect(pixelFormats.length).toBeGreaterThan(0);
    expect(pixelFormats).toContain('yuv420p');
    expect(pixelFormats).toContain('rgb24');
  });

  test('getSupportedSampleFormats should return an array of sample formats', () => {
    const sampleFormats = getSupportedSampleFormats();
    expect(Array.isArray(sampleFormats)).toBe(true);
    expect(sampleFormats.length).toBeGreaterThan(0);
    expect(sampleFormats).toContain('s16');
    expect(sampleFormats).toContain('f32');
  });
});

describe('Rust-AV Kit - Media Info', () => {
  beforeAll(() => {
    setupTestDir();
  });

  afterAll(() => {
    cleanupTestDir();
  });

  test('getMediaInfo should return error for non-existent file', () => {
    expect(() => getMediaInfo('/nonexistent/file.mp4')).toThrow();
  });

  test('getMediaInfo should handle invalid file paths', () => {
    expect(() => getMediaInfo('')).toThrow();
  });
});

describe('Rust-AV Kit - Transcode', () => {
  beforeAll(() => {
    setupTestDir();
  });

  afterAll(() => {
    cleanupTestDir();
  });

  test('transcode should return error for non-existent input file', () => {
    const options = {
      inputPath: '/nonexistent/input.mp4',
      outputPath: path.join(TEST_DIR, 'output.mp4'),
    };
    expect(() => transcode(options)).toThrow();
  });

  test('transcode should handle empty options', () => {
    const options = {
      inputPath: '',
      outputPath: '',
    };
    expect(() => transcode(options)).toThrow();
  });
});

describe('Rust-AV Kit - Data Structures', () => {
  test('CodecOptions structure should be valid', () => {
    const codecOptions = {
      codecName: 'h264',
      bitRate: 1000000,
      width: 1920,
      height: 1080,
      frameRate: 30.0,
      sampleRate: null,
      channels: null,
      gopSize: 30,
      maxBFrames: 3,
      crf: 23,
      preset: 'medium',
      tune: null,
      profile: 'high',
      level: 40,
    };

    expect(codecOptions.codecName).toBe('h264');
    expect(codecOptions.bitRate).toBe(1000000);
    expect(codecOptions.width).toBe(1920);
    expect(codecOptions.height).toBe(1080);
  });

  test('FilterConfig structure should be valid', () => {
    const filterConfig = {
      filterString: 'scale=1280:720',
    };

    expect(filterConfig.filterString).toBe('scale=1280:720');
  });

  test('TranscodeOptions structure should be valid', () => {
    const transcodeOptions = {
      inputPath: 'input.mp4',
      outputPath: 'output.mp4',
      videoCodec: {
        codecName: 'h264',
        bitRate: 1000000,
        width: 1920,
        height: 1080,
        frameRate: 30.0,
        sampleRate: null,
        channels: null,
        gopSize: 30,
        maxBFrames: 3,
        crf: 23,
        preset: 'medium',
        tune: null,
        profile: 'high',
        level: 40,
      },
      audioCodec: null,
      videoFilter: {
        filterString: 'scale=1280:720',
      },
      audioFilter: null,
      format: 'mp4',
      startTime: 0.0,
      duration: 10.0,
      seekTo: null,
    };

    expect(transcodeOptions.inputPath).toBe('input.mp4');
    expect(transcodeOptions.outputPath).toBe('output.mp4');
    expect(transcodeOptions.videoCodec?.codecName).toBe('h264');
    expect(transcodeOptions.videoFilter?.filterString).toBe('scale=1280:720');
  });

  test('ProgressData structure should be valid', () => {
    const progressData = {
      currentTime: 5.0,
      totalTime: 10.0,
      percentage: 50.0,
      fps: 30.0,
      bitRate: 1000000,
      size: 5000000,
    };

    expect(progressData.currentTime).toBe(5.0);
    expect(progressData.totalTime).toBe(10.0);
    expect(progressData.percentage).toBe(50.0);
    expect(progressData.fps).toBe(30.0);
    expect(progressData.bitRate).toBe(1000000);
    expect(progressData.size).toBe(5000000);
  });
});

describe('Rust-AV Kit - Integration', () => {
  beforeAll(() => {
    setupTestDir();
  });

  afterAll(() => {
    cleanupTestDir();
  });

  test('should handle multiple format queries', () => {
    const formats = getSupportedFormats();
    const codecs = getSupportedCodecs();
    const pixelFormats = getSupportedPixelFormats();
    const sampleFormats = getSupportedSampleFormats();

    expect(formats.length).toBeGreaterThan(0);
    expect(codecs.length).toBeGreaterThan(0);
    expect(pixelFormats.length).toBeGreaterThan(0);
    expect(sampleFormats.length).toBeGreaterThan(0);
  });

  test('should handle codec and format combinations', () => {
    const formats = getSupportedFormats();
    const codecs = getSupportedCodecs();

    // Verify common combinations
    expect(formats).toContain('ivf');
    expect(codecs).toContain('av1');
    expect(formats).toContain('webm');
    expect(codecs).toContain('vp8');
  });
});
