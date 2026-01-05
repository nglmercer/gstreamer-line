/**
 * Fetch Test Videos Script
 *
 * This script downloads sample videos in different formats (Y4M, IVF, MKV)
 * from public sources for testing and examples.
 *
 * Usage:
 *   - Fetch videos: bun run examples/fetch_test_videos.ts
 *   - Clean up: bun run examples/fetch_test_videos.ts --clean
 */

import * as fs from 'node:fs';
import * as path from 'node:path';

// ============================================================================
// Configuration
// ============================================================================

const TEST_DIR = path.join(__dirname, 'test_files');

// Test video configurations for generation using FFmpeg
// This approach is more reliable than downloading from external sources
const VIDEO_SOURCES = {
  // Y4M format videos (shorter duration due to uncompressed format)
  y4m: [
    {
      name: 'sample_320x240.y4m',
      description: 'Sample Y4M video - 320x240 (5s)',
      width: 320,
      height: 240,
      duration: 5,
      fps: 30,
    },
    {
      name: 'sample_640x360.y4m',
      description: 'Sample Y4M video - 640x360 (5s)',
      width: 640,
      height: 360,
      duration: 5,
      fps: 30,
    },
  ],
  // IVF format videos (VP8/VP9)
  ivf: [
    {
      name: 'sample_320x240.ivf',
      description: 'Sample IVF video - 320x240 VP8 (30s)',
      width: 320,
      height: 240,
      duration: 30,
      fps: 30,
      codec: 'libvpx',
    },
  ],
  // Matroska (MKV) format videos
  mkv: [
    {
      name: 'sample_640x480.mkv',
      description: 'Sample MKV video - 640x480 (30s)',
      width: 640,
      height: 480,
      duration: 30,
      fps: 30,
      codec: 'libx264',
    },
  ],
};

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Ensure the test directory exists
 */
function ensureTestDir(): void {
  if (!fs.existsSync(TEST_DIR)) {
    fs.mkdirSync(TEST_DIR, { recursive: true });
    console.log(`Created test directory: ${TEST_DIR}`);
  }
}

/**
 * Check if FFmpeg is available for format conversion
 */
function checkFFmpegAvailable(): boolean {
  try {
    require('child_process').execSync('ffmpeg -version', { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

/**
 * Execute FFmpeg command
 */
async function executeFFmpeg(command: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const { exec } = require('child_process');

    exec(command, (error: any, stdout: string, stderr: string) => {
      if (error) {
        reject(new Error(`FFmpeg execution failed: ${error.message}`));
        return;
      }
      resolve();
    });
  });
}

/**
 * Fetch all videos
 */
async function fetchVideos(): Promise<void> {
  console.log('=== Fetching Test Videos ===\n');
  ensureTestDir();

  const ffmpegAvailable = checkFFmpegAvailable();
  if (!ffmpegAvailable) {
    console.warn('⚠ FFmpeg not found. Format conversion will be skipped.');
    console.warn('  Install FFmpeg to enable conversion: https://ffmpeg.org/download.html\n');
  }

  let totalDownloaded = 0;
  let totalConverted = 0;

  // Generate Y4M videos using FFmpeg
  if (ffmpegAvailable) {
    console.log('\n--- Generating Y4M Videos ---\n');
    for (const video of VIDEO_SOURCES.y4m) {
      const outputPath = path.join(TEST_DIR, video.name);
      if (fs.existsSync(outputPath)) {
        console.log(`✓ Already exists: ${video.name}`);
        totalDownloaded++;
        continue;
      }

      try {
        // Use color source instead of testsrc for better Y4M compatibility
        const ffmpegCommand = `ffmpeg -f lavfi -i "testsrc=size=${video.width}x${video.height}:duration=${video.duration}:rate=${video.fps}" -pix_fmt yuv420p -y "${outputPath}"`;        await executeFFmpeg(ffmpegCommand);
        console.log(`  ${video.description}`);
        totalDownloaded++;
      } catch (error) {
        console.error(`✗ Failed to generate ${video.name}:`, (error as Error).message);
      }
    }
  } else {
    console.log('\n--- Skipping Y4M Videos (FFmpeg not available) ---\n');
  }

  // Generate IVF videos using FFmpeg
  if (ffmpegAvailable) {
    console.log('\n--- Generating IVF Videos ---\n');
    for (const video of VIDEO_SOURCES.ivf) {
      const outputPath = path.join(TEST_DIR, video.name);
      if (fs.existsSync(outputPath)) {
        console.log(`✓ Already exists: ${video.name}`);
        totalDownloaded++;
        continue;
      }

      try {
        const codec = (video as any).codec || 'libvpx';
        const ffmpegCommand = `ffmpeg -f lavfi -i testsrc=size=${video.width}x${video.height}:duration=${video.duration}:rate=${video.fps} -c:v ${codec} -b:v 1M -f ivf -y "${outputPath}"`;
        await executeFFmpeg(ffmpegCommand);
        console.log(`  ${video.description}`);
        totalDownloaded++;
      } catch (error) {
        console.error(`✗ Failed to generate ${video.name}:`, (error as Error).message);
      }
    }
  } else {
    console.log('\n--- Skipping IVF Videos (FFmpeg not available) ---\n');
  }

  // Generate MKV videos using FFmpeg
  if (ffmpegAvailable) {
    console.log('\n--- Generating Matroska (MKV) Videos ---\n');
    for (const video of VIDEO_SOURCES.mkv) {
      const outputPath = path.join(TEST_DIR, video.name);
      if (fs.existsSync(outputPath)) {
        console.log(`✓ Already exists: ${video.name}`);
        totalDownloaded++;
        continue;
      }

      try {
        const codec = (video as any).codec || 'libx264';
        const ffmpegCommand = `ffmpeg -f lavfi -i testsrc=size=${video.width}x${video.height}:duration=${video.duration}:rate=${video.fps} -c:v ${codec} -b:v 1M -f matroska -y "${outputPath}"`;
        await executeFFmpeg(ffmpegCommand);
        console.log(`  ${video.description}`);
        totalDownloaded++;
      } catch (error) {
        console.error(`✗ Failed to generate ${video.name}:`, (error as Error).message);
      }
    }
  } else {
    console.log('\n--- Skipping MKV Videos (FFmpeg not available) ---\n');
  }

  // Summary
  console.log('\n=== Summary ===');
  console.log(`Total videos downloaded: ${totalDownloaded}`);
  console.log(`Total videos converted: ${totalConverted}`);
  console.log(`Total videos available: ${totalDownloaded + totalConverted}`);
}

/**
 * Clean up all test videos
 */
function cleanupTestVideos(): void {
  console.log('=== Cleaning Up Test Videos ===\n');

  if (!fs.existsSync(TEST_DIR)) {
    console.log('No test directory found. Nothing to clean.');
    return;
  }

  const files = fs.readdirSync(TEST_DIR);
  if (files.length === 0) {
    console.log('Test directory is empty. Nothing to clean.');
    return;
  }

  let deletedCount = 0;
  let totalSize = 0;

  for (const file of files) {
    const filePath = path.join(TEST_DIR, file);
    const stats = fs.statSync(filePath);

    try {
      fs.unlinkSync(filePath);
      totalSize += stats.size;
      deletedCount++;
      console.log(`✓ Deleted: ${file} (${(stats.size / 1024).toFixed(2)} KB)`);
    } catch (error) {
      console.error(`✗ Failed to delete ${file}:`, (error as Error).message);
    }
  }

  // Remove the test directory if empty
  const remainingFiles = fs.readdirSync(TEST_DIR);
  if (remainingFiles.length === 0) {
    fs.rmdirSync(TEST_DIR);
    console.log(`✓ Removed empty test directory: ${TEST_DIR}`);
  }

  console.log('\n=== Summary ===');
  console.log(`Files deleted: ${deletedCount}`);
  console.log(`Total space freed: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);
}

/**
 * List available test videos
 */
function listTestVideos(): void {
  console.log('=== Available Test Videos ===\n');

  if (!fs.existsSync(TEST_DIR)) {
    console.log('No test directory found. Run the script to fetch videos.');
    return;
  }

  const files = fs.readdirSync(TEST_DIR);
  if (files.length === 0) {
    console.log('Test directory is empty. Run the script to fetch videos.');
    return;
  }

  let totalSize = 0;
  const formatCounts: Record<string, number> = {};

  for (const file of files) {
    const filePath = path.join(TEST_DIR, file);
    const stats = fs.statSync(filePath);
    const ext = path.extname(file).slice(1).toUpperCase();

    totalSize += stats.size;
    formatCounts[ext] = (formatCounts[ext] || 0) + 1;

    console.log(`  ${file} (${(stats.size / 1024).toFixed(2)} KB)`);
  }

  console.log('\n--- Summary ---');
  console.log(`Total files: ${files.length}`);
  console.log(`Total size: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);
  console.log('Formats:');
  for (const [format, count] of Object.entries(formatCounts)) {
    console.log(`  ${format}: ${count} file(s)`);
  }
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0] || 'fetch';

  try {
    switch (command) {
      case 'fetch':
      case 'download':
        await fetchVideos();
        break;
      case 'clean':
      case 'cleanup':
        cleanupTestVideos();
        break;
      case 'list':
      case 'ls':
        listTestVideos();
        break;
      case 'help':
      case '--help':
      case '-h':
        console.log('Fetch Test Videos Script');
        console.log('\nUsage:');
        console.log('  bun run examples/fetch_test_videos.ts [command]');
        console.log('\nCommands:');
        console.log('  fetch, download  Download sample videos (default)');
        console.log('  clean, cleanup   Remove all test videos');
        console.log('  list, ls         List available test videos');
        console.log('  help, --help     Show this help message');
        break;
      default:
        console.error(`Unknown command: ${command}`);
        console.log('Run with --help for usage information');
        process.exit(1);
    }
  } catch (error) {
    console.error('Error:', (error as Error).message);
    process.exit(1);
  }
}

// Run the script
main();
