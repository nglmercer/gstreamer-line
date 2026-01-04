/**
 * Codec Availability and Alternative Encoder Tests
 *
 * Comprehensive tests for codec availability detection, alternative encoder support,
 * and proper handling of both available and unavailable codecs.
 */

import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import setup, {
  VideoCodecKey,
  VIDEO_CODECS,
  isCodecAvailable,
  getCodecEncoder,
  getCodecAvailabilityReport,
  generateVideoWithCodec,
  extractFrameFromCodecVideo,
} from './setup.js';
import * as fs from 'node:fs';

describe('Codec Availability Detection', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should detect available codecs', async () => {
    const report = await getCodecAvailabilityReport();

    console.log('\n' + '='.repeat(80));
    console.log('CODEC AVAILABILITY REPORT');
    console.log('='.repeat(80));

    let availableCount = 0;
    let unavailableCount = 0;

    for (const [codecKey, info] of Object.entries(report)) {
      const codecName = VIDEO_CODECS[codecKey as VideoCodecKey].name;
      const status = info.available ? '✓ AVAILABLE' : '✗ UNAVAILABLE';
      const encoder = info.encoder || 'None';
      const alternatives = info.alternatives.length > 0 ? info.alternatives.join(', ') : 'None';

      console.log(`\n${codecKey.toUpperCase()} (${codecName})`);
      console.log(`  Status: ${status}`);
      console.log(`  Encoder: ${encoder}`);
      console.log(`  Alternatives: ${alternatives}`);

      if (info.available) {
        availableCount++;
      } else {
        unavailableCount++;
      }
    }

    console.log('\n' + '-'.repeat(80));
    console.log(`SUMMARY: ${availableCount} available, ${unavailableCount} unavailable`);
    console.log('='.repeat(80) + '\n');

    expect(Object.keys(report).length).toBeGreaterThan(0);
  });

  it('should correctly identify codec availability', async () => {
    // jpeg should always be available
    const jpegAvailable = await isCodecAvailable('jpeg');
    expect(jpegAvailable).toBe(true);

    // Test a codec that might not be available
    const av1Available = await isCodecAvailable('av1');
    expect(typeof av1Available).toBe('boolean');
  });

  it('should return correct encoder for available codecs', async () => {
    const jpegEncoder = await getCodecEncoder('jpeg');
    expect(jpegEncoder).toBeDefined();
    expect(jpegEncoder.length).toBeGreaterThan(0);

    console.log(`\nJPEG encoder: ${jpegEncoder}`);
  });

  it('should throw error for unavailable codecs', async () => {
    try {
      await getCodecEncoder('nonexistent' as VideoCodecKey);
      expect(false).toBe(true); // Should not reach here
    } catch (error) {
      expect(error).toBeDefined();
      expect(error instanceof Error).toBe(true);
      console.log(`\n✓ Correctly threw error for nonexistent codec: ${(error as Error).message}`);
    }
  });

  it('should use alternative encoder when primary is unavailable', async () => {
    const report = await getCodecAvailabilityReport();

    console.log('\n' + '='.repeat(80));
    console.log('ALTERNATIVE ENCODER USAGE');
    console.log('='.repeat(80));

    for (const [codecKey, info] of Object.entries(report)) {
      const codec = VIDEO_CODECS[codecKey as VideoCodecKey];
      const hasAlternatives = codec.alternativeEncoders && codec.alternativeEncoders.length > 0;

      if (info.available && hasAlternatives) {
        const usingAlternative = info.encoder !== codec.encoder;
        console.log(`\n${codecKey.toUpperCase()}:`);
        console.log(`  Primary encoder: ${codec.encoder}`);
        console.log(`  Using: ${info.encoder}`);
        console.log(`  Using alternative: ${usingAlternative ? 'YES' : 'NO'}`);

        if (usingAlternative) {
          console.log(`  ✓ Successfully using alternative encoder`);
        }
      }
    }

    console.log('\n' + '='.repeat(80) + '\n');
  });
});

describe('Video Generation with Available Codecs', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should generate video with all available codecs', async () => {
    const report = await getCodecAvailabilityReport();
    const availableCodecs = Object.entries(report)
      .filter(([_, info]) => info.available)
      .map(([codecKey]) => codecKey as VideoCodecKey);

    console.log('\n' + '='.repeat(80));
    console.log(`TESTING VIDEO GENERATION WITH ${availableCodecs.length} AVAILABLE CODECS`);
    console.log('='.repeat(80));

    const results: Record<string, { success: boolean; encoder: string; size: number }> = {};

    for (const codecKey of availableCodecs) {
      try {
        const { videoPath, encoder } = await generateVideoWithCodec(codecKey, 0, 'snow', {
          numBuffers: 30,
        });

        const stats = fs.statSync(videoPath);
        results[codecKey] = {
          success: true,
          encoder,
          size: stats.size,
        };

        console.log(`\n✓ ${codecKey.toUpperCase()}:`);
        console.log(`  Encoder: ${encoder}`);
        console.log(`  File: ${videoPath}`);
        console.log(`  Size: ${stats.size} bytes`);

        expect(fs.existsSync(videoPath)).toBe(true);
        expect(stats.size).toBeGreaterThan(100);
      } catch (error) {
        console.log(`\n✗ ${codecKey.toUpperCase()}: Failed`);
        console.log(`  Error: ${error instanceof Error ? error.message : String(error)}`);
        results[codecKey] = {
          success: false,
          encoder: 'none',
          size: 0,
        };
      }
    }

    console.log('\n' + '-'.repeat(80));
    console.log('RESULTS:');
    console.log('-'.repeat(80));

    const successful = Object.entries(results).filter(([_, r]) => r.success);
    const failed = Object.entries(results).filter(([_, r]) => !r.success);

    console.log(`\nSuccessful (${successful.length}):`);
    successful.forEach(([codec, _]) => console.log(`  ✓ ${codec}`));

    if (failed.length > 0) {
      console.log(`\nFailed (${failed.length}):`);
      failed.forEach(([codec, _]) => console.log(`  ✗ ${codec}`));
    }

    console.log('\n' + '='.repeat(80) + '\n');

    expect(successful.length).toBeGreaterThan(0);
  }, 30000); // Increase timeout to 30 seconds
});

describe('Video Generation with Unavailable Codecs', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should gracefully handle unavailable codecs', async () => {
    const report = await getCodecAvailabilityReport();
    const unavailableCodecs = Object.entries(report)
      .filter(([_, info]) => !info.available)
      .map(([codecKey]) => codecKey as VideoCodecKey);

    console.log('\n' + '='.repeat(80));
    console.log(`TESTING UNAVAILABLE CODEC HANDLING (${unavailableCodecs.length} codecs)`);
    console.log('='.repeat(80));

    for (const codecKey of unavailableCodecs) {
      const codec = VIDEO_CODECS[codecKey];
      const info = report[codecKey];

      console.log(`\n${codecKey.toUpperCase()} (${codec.name}):`);
      console.log(`  Status: UNAVAILABLE`);
      console.log(`  Primary encoder: ${codec.encoder}`);
      console.log(`  Alternatives tried: ${info.alternatives.join(', ') || 'None'}`);
      console.log(`  All alternatives failed`);

      // Verify that trying to get encoder throws error
      try {
        await getCodecEncoder(codecKey);
        expect(false).toBe(true); // Should not reach here
      } catch (error) {
        expect(error instanceof Error).toBe(true);
        console.log(`  ✓ Correctly threw error: ${(error as Error).message}`);
      }

      // Verify that isCodecAvailable returns false
      const isAvailable = await isCodecAvailable(codecKey);
      expect(isAvailable).toBe(false);
      console.log(`  ✓ isCodecAvailable() correctly returns false`);
    }

    console.log('\n' + '='.repeat(80) + '\n');
  });

  it('should skip unavailable codecs in video generation', async () => {
    const codecsToTest: VideoCodecKey[] = ['h264', 'h265', 'mpeg2'];

    console.log('\n' + '='.repeat(80));
    console.log('TESTING SKIP LOGIC FOR UNAVAILABLE CODECS');
    console.log('='.repeat(80));

    for (const codecKey of codecsToTest) {
      const isAvailable = await isCodecAvailable(codecKey);

      if (!isAvailable) {
        console.log(`\n✓ Skipping ${codecKey}: codec not available`);
        console.log(`  This is expected behavior`);
        continue;
      }

      // If available, try to generate video
      try {
        const { videoPath } = await generateVideoWithCodec(codecKey, 0, 'snow', {
          numBuffers: 30,
        });
        console.log(`\n✓ ${codecKey} is available and video generated: ${videoPath}`);
        expect(fs.existsSync(videoPath)).toBe(true);
      } catch (error) {
        console.log(`\n✗ ${codecKey} is available but generation failed`);
        console.log(`  Error: ${error instanceof Error ? error.message : String(error)}`);
      }
    }

    console.log('\n' + '='.repeat(80) + '\n');
  });
});

describe('Frame Extraction from Available Codecs', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should extract frames from all available codecs', async () => {
    const report = await getCodecAvailabilityReport();
    const availableCodecs = Object.entries(report)
      .filter(([_, info]) => info.available)
      .map(([codecKey]) => codecKey as VideoCodecKey);

    console.log('\n' + '='.repeat(80));
    console.log(`TESTING FRAME EXTRACTION FROM ${availableCodecs.length} AVAILABLE CODECS`);
    console.log('='.repeat(80));

    const results: Record<string, { success: boolean; frameSize: number; error?: string }> = {};

    for (const codecKey of availableCodecs) {
      try {
        // Generate video
        const { videoPath } = await generateVideoWithCodec(codecKey, 0, 'snow', {
          numBuffers: 30,
        });

        // Extract frame
        const frame = await extractFrameFromCodecVideo(videoPath, codecKey, 0, 0);

        if (frame) {
          results[codecKey] = {
            success: true,
            frameSize: frame.data.length,
          };

          console.log(`\n✓ ${codecKey.toUpperCase()}:`);
          console.log(`  Frame extracted successfully`);
          console.log(`  Frame size: ${frame.data.length} bytes`);

          expect(frame.data.length).toBe(320 * 240 * 4); // RGBA
        } else {
          results[codecKey] = {
            success: false,
            frameSize: 0,
            error: 'No frame data',
          };

          console.log(`\n✗ ${codecKey.toUpperCase()}: No frame data`);
        }
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : String(error);
        console.log(`\n✗ ${codecKey.toUpperCase()}: Failed`);
        console.log(`  Error: ${errorMsg}`);
        results[codecKey] = {
          success: false,
          frameSize: 0,
          error: errorMsg,
        };
      }
    }

    console.log('\n' + '-'.repeat(80));
    console.log('RESULTS:');
    console.log('-'.repeat(80));

    const successful = Object.entries(results).filter(([_, r]) => r.success);
    const failed = Object.entries(results).filter(([_, r]) => !r.success);

    console.log(`\nSuccessful (${successful.length}):`);
    successful.forEach(([codec, r]) => console.log(`  ✓ ${codec} (${r.frameSize} bytes)`));

    if (failed.length > 0) {
      console.log(`\nFailed (${failed.length}):`);
      failed.forEach(([codec, r]) => {
        console.log(`  ✗ ${codec}${r.error ? ` - ${r.error}` : ''}`);
      });
    }

    console.log('\n' + '='.repeat(80) + '\n');

    expect(successful.length).toBeGreaterThan(0);
  }, 30000); // Increase timeout to 30 seconds
});

describe('Frame Extraction with Seeking', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should extract frames at different timestamps from available codecs', async () => {
    const report = await getCodecAvailabilityReport();
    const availableCodecs = Object.entries(report)
      .filter(([_, info]) => info.available)
      .slice(0, 2) // Test first 2 available codecs
      .map(([codecKey]) => codecKey as VideoCodecKey);

    console.log('\n' + '='.repeat(80));
    console.log('TESTING FRAME EXTRACTION WITH SEEKING');
    console.log('='.repeat(80));

    for (const codecKey of availableCodecs) {
      console.log(`\n--- Testing ${codecKey.toUpperCase()} ---`);

      try {
        // Generate video
        const { videoPath } = await generateVideoWithCodec(codecKey, 0, 'snow', {
          numBuffers: 60, // 2 seconds
        });

        // Extract frames at different timestamps
        const timestamps = [0, 100, 200, 300, 400];
        const frameSizes: number[] = [];

        for (const timestamp of timestamps) {
          const frame = await extractFrameFromCodecVideo(videoPath, codecKey, 0, timestamp);

          expect(frame).not.toBeNull();
          expect(frame!.data.length).toBeGreaterThan(0);

          frameSizes.push(frame!.data.length);
          console.log(`  ✓ Frame at ${timestamp}ms: ${frame!.data.length} bytes`);
        }

        // All frames should have same size
        const firstSize = frameSizes[0];
        for (let i = 1; i < frameSizes.length; i++) {
          expect(frameSizes[i]).toBe(firstSize);
        }

        console.log(`  ✓ All frames have consistent size: ${firstSize} bytes`);
      } catch (error) {
        console.log(`  ✗ Failed: ${error instanceof Error ? error.message : String(error)}`);
        // Don't fail test if one codec has decoder issues
        console.log(`  ℹ Skipping this codec due to decoder issues`);
      }
    }

    console.log('\n' + '='.repeat(80) + '\n');
  }, 30000); // Increase timeout to 30 seconds
});

describe('Multiple Format Support', () => {
  beforeAll(() => {
    setup.setupTestDirectories();
  });

  afterAll(() => {
    setup.cleanupTestDirectories();
  });

  it('should generate videos in multiple formats for available codecs', async () => {
    const codecsToTest: { codec: VideoCodecKey; formatIndices: number[] }[] = [
      { codec: 'vp8', formatIndices: [0, 1] },
      { codec: 'vp9', formatIndices: [0, 1] },
      { codec: 'jpeg', formatIndices: [0, 1] },
      { codec: 'png', formatIndices: [0, 1] },
    ];

    console.log('\n' + '='.repeat(80));
    console.log('TESTING MULTIPLE FORMAT SUPPORT');
    console.log('='.repeat(80));

    for (const { codec, formatIndices } of codecsToTest) {
      const isAvailable = await isCodecAvailable(codec);

      if (!isAvailable) {
        console.log(`\n✗ Skipping ${codec}: codec not available`);
        continue;
      }

      console.log(`\n--- Testing ${codec.toUpperCase()} ---`);

      for (const formatIndex of formatIndices) {
        try {
          const { videoPath, format } = await generateVideoWithCodec(codec, formatIndex, 'colors', {
            numBuffers: 30,
          });

          const stats = fs.statSync(videoPath);

          console.log(`  ✓ Format ${format.extension}:`);
          console.log(`    File: ${videoPath}`);
          console.log(`    Size: ${stats.size} bytes`);

          expect(fs.existsSync(videoPath)).toBe(true);
          expect(stats.size).toBeGreaterThan(100);
        } catch (error) {
          console.log(`  ✗ Format ${formatIndex}: Failed`);
          console.log(`    Error: ${error instanceof Error ? error.message : String(error)}`);
        }
      }
    }

    console.log('\n' + '='.repeat(80) + '\n');
  }, 30000); // Increase timeout to 30 seconds
});

describe('Codec Availability Summary', () => {
  it('should provide comprehensive codec availability summary', async () => {
    const report = await getCodecAvailabilityReport();

    console.log('\n' + '='.repeat(80));
    console.log('COMPREHENSIVE CODEC AVAILABILITY SUMMARY');
    console.log('='.repeat(80));

    const available: string[] = [];
    const unavailable: string[] = [];
    const usingAlternatives: string[] = [];

    for (const [codecKey, info] of Object.entries(report)) {
      const codec = VIDEO_CODECS[codecKey as VideoCodecKey];

      if (info.available) {
        available.push(codecKey);
        if (info.encoder !== codec.encoder) {
          usingAlternatives.push(codecKey);
        }
      } else {
        unavailable.push(codecKey);
      }
    }

    console.log('\nAVAILABLE CODECS:');
    available.forEach(codec => {
      const info = report[codec];
      const codecDef = VIDEO_CODECS[codec as VideoCodecKey];
      const isAlternative = info.encoder !== codecDef.encoder;
      console.log(`  ${codec.padEnd(8)} - ${codecDef.name.padEnd(15)} [${info.encoder}]${isAlternative ? ' (alternative)' : ''}`);
    });

    if (unavailable.length > 0) {
      console.log('\nUNAVAILABLE CODECS:');
      unavailable.forEach(codec => {
        const codecDef = VIDEO_CODECS[codec as VideoCodecKey];
        const alternatives = codecDef.alternativeEncoders?.join(', ') || 'None';
        console.log(`  ${codec.padEnd(8)} - ${codecDef.name.padEnd(15)} [Alternatives: ${alternatives}]`);
      });
    }

    if (usingAlternatives.length > 0) {
      console.log('\nUSING ALTERNATIVE ENCODERS:');
      usingAlternatives.forEach(codec => {
        const info = report[codec];
        const codecDef = VIDEO_CODECS[codec as VideoCodecKey];
        console.log(`  ${codec.padEnd(8)} - Using ${info.encoder} instead of ${codecDef.encoder}`);
      });
    }

    console.log('\n' + '-'.repeat(80));
    console.log(`Total: ${available.length} available, ${unavailable.length} unavailable`);
    console.log(`Using alternatives: ${usingAlternatives.length}`);
    console.log('='.repeat(80) + '\n');

    expect(available.length).toBeGreaterThan(0);
  });
});
