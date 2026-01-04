/**
 * Basic Usage Example for Rust-AV Kit
 * 
 * This example demonstrates basic usage of the Rust-AV Kit library
 * for media transcoding, format transformation, and media info extraction.
 */

import { 
  getMediaInfo, 
  transcode, 
  transformFormat,
  getSupportedFormats,
  getSupportedCodecs,
  getSupportedPixelFormats,
  getSupportedSampleFormats,
} from '../index.js';

import * as fs from 'node:fs';
import * as path from 'node:path';

// ============================================================================
// Helper Functions
// ============================================================================

function createTestFile(filename: string, content: string): string {
  const testDir = path.join(__dirname, 'test_files');
  if (!fs.existsSync(testDir)) {
    fs.mkdirSync(testDir, { recursive: true });
  }
  const filePath = path.join(testDir, filename);
  fs.writeFileSync(filePath, content);
  return filePath;
}

function cleanupTestFiles() {
  const testDir = path.join(__dirname, 'test_files');
  if (fs.existsSync(testDir)) {
    const files = fs.readdirSync(testDir);
    for (const file of files) {
      fs.unlinkSync(path.join(testDir, file));
    }
  }
}

// ============================================================================
// Example 1: Get Supported Formats and Codecs
// ============================================================================

console.log('=== Example 1: Get Supported Formats and Codecs ===\n');

const formats = getSupportedFormats();
console.log('Supported formats:', formats);

const codecs = getSupportedCodecs();
console.log('Supported codecs:', codecs);

const pixelFormats = getSupportedPixelFormats();
console.log('Supported pixel formats:', pixelFormats);

const sampleFormats = getSupportedSampleFormats();
console.log('Supported sample formats:', sampleFormats);

// ============================================================================
// Example 2: Get Media Information
// ============================================================================

console.log('\n=== Example 2: Get Media Information ===\n');

// Create a simple test Y4M file
const y4mContent = 'YUV4MPEG2 W320 H240 F30:1 Ip A1:1 C420mpeg2\n' +
  'FRAME\n' +
  Buffer.alloc(320 * 240, 128).toString('binary') + // Y plane
  Buffer.alloc(320 * 240 / 4, 128).toString('binary') + // U plane
  Buffer.alloc(320 * 240 / 4, 128).toString('binary'); // V plane

const y4mPath = createTestFile('test_video.y4m', y4mContent);

// Get media information from the file
try {
  const mediaInfo = getMediaInfo(y4mPath);
  console.log('Media Info:', JSON.stringify(mediaInfo, null, 2));
  console.log(`  Format: ${mediaInfo.format.name} (${mediaInfo.format.longName})`);
  console.log(`  Duration: ${mediaInfo.format.duration?.toFixed(2) || 'N/A'}s`);
  console.log(`  Bitrate: ${mediaInfo.format.bitRate || 'N/A'} bps`);
  console.log(`  Streams: ${mediaInfo.streams.length}`);
  
  if (mediaInfo.streams.length > 0) {
    const stream = mediaInfo.streams[0];
    console.log(`  Stream 0:`);
    console.log(`    Codec: ${stream.codecName} (${stream.codecType})`);
    console.log(`    Dimensions: ${stream.width || 'N/A'}x${stream.height || 'N/A'}`);
    console.log(`    Frame Rate: ${stream.frameRate || 'N/A'} fps`);
  }
} catch (error) {
  console.error('Error getting media info:', error);
}

// ============================================================================
// Example 3: Format Transformation
// ============================================================================

console.log('\n=== Example 3: Format Transformation ===\n');

// Transform Y4M to IVF
const ivfPath = path.join(__dirname, 'test_files', 'test_video.ivf');
try {
  transformFormat(y4mPath, ivfPath);
  console.log(`Transformed ${y4mPath} to ${ivfPath}`);
  
  // Verify the output file was created
  if (fs.existsSync(ivfPath)) {
    const stats = fs.statSync(ivfPath);
    console.log(`  Output file size: ${stats.size} bytes`);
  }
} catch (error) {
  console.error('Error transforming format:', error);
}

// ============================================================================
// Example 4: Advanced Transcoding with Options
// ============================================================================

console.log('\n=== Example 4: Advanced Transcoding with Options ===\n');

// Create a test Y4M file for transcoding
const y4mPath2 = createTestFile('test_video2.y4m', y4mContent);

// Transcode with codec options
const matroskaPath = path.join(__dirname, 'test_files', 'test_video.mkv');
try {
  transcode({
    inputPath: y4mPath2,
    outputPath: matroskaPath,
    videoCodec: {
      codecName: 'av1',
      width: 640,
      height: 480,
      frameRate: 30.0,
      bitRate: 1000000,
    },
    audioCodec: undefined,
    videoFilter: undefined,
    audioFilter: undefined,
    format: 'matroska',
    startTime: undefined,
    duration: undefined,
    seekTo: undefined,
  });
  console.log(`Transcoded ${y4mPath2} to ${matroskaPath} with codec options`);
  
  // Verify the output file was created
  if (fs.existsSync(matroskaPath)) {
    const stats = fs.statSync(matroskaPath);
    console.log(`  Output file size: ${stats.size} bytes`);
  }
} catch (error) {
  console.error('Error transcoding:', error);
}

// ============================================================================
// Example 5: Transcoding with Video Filters
// ============================================================================

console.log('\n=== Example 5: Transcoding with Video Filters ===\n');

// Create a test Y4M file for filtering
const y4mPath3 = createTestFile('test_video3.y4m', y4mContent);

// Transcode with scale filter
const filteredPath = path.join(__dirname, 'test_files', 'test_video_scaled.mkv');
try {
  transcode({
    inputPath: y4mPath3,
    outputPath: filteredPath,
    videoCodec: undefined,
    audioCodec: undefined,
    videoFilter: {
      filterString: 'scale=640:480',
    },
    audioFilter: undefined,
    format: undefined,
    startTime: undefined,
    duration: undefined,
    seekTo: undefined,
  });
  console.log(`Transcoded ${y4mPath3} to ${filteredPath} with scale filter`);
  
  // Verify the output file was created
  if (fs.existsSync(filteredPath)) {
    const stats = fs.statSync(filteredPath);
    console.log(`  Output file size: ${stats.size} bytes`);
  }
} catch (error) {
  console.error('Error transcoding with filter:', error);
}

// ============================================================================
// Example 6: Error Handling
// ============================================================================

console.log('\n=== Example 6: Error Handling ===\n');

// Try to get media info from non-existent file
try {
  getMediaInfo('/non/existent/file.mp4');
  console.log('This should not print');
} catch (error) {
  console.log('Expected error caught:', (error as Error).message);
}

// Try to transform non-existent file
try {
  transformFormat('/non/existent/input.y4m', '/non/existent/output.mkv');
  console.log('This should not print');
} catch (error) {
  console.log('Expected error caught:', (error as Error).message);
}

// ============================================================================
// Cleanup
// ============================================================================

console.log('\n=== Cleanup ===\n');
cleanupTestFiles();

console.log('\n=== Examples Complete ===\n');
console.log('All examples have been executed successfully.');
console.log('Check the test_files directory for generated output files.');
