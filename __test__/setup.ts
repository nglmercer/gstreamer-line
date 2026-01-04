/**
 * Test Setup Utilities for Rust-AV Kit
 *
 * This file provides reusable utilities for:
 * - Creating test media files (IVF, Matroska, Y4M)
 * - Common test setup/teardown
 * - Media validation helpers
 */

import * as fs from 'node:fs';
import * as path from 'node:path';

// ============================================================================
// Configuration
// ============================================================================

export const TEST_DIR = path.join(__dirname, 'temp_output');

export interface MediaConfig {
  width: number;
  height: number;
  framerate: number;
  duration: number; // in seconds
}

export const DEFAULT_MEDIA_CONFIG: MediaConfig = {
  width: 320,
  height: 240,
  framerate: 30,
  duration: 1,
};

// ============================================================================
// Directory Management
// ============================================================================

export function setupTestDirectories() {
  if (!fs.existsSync(TEST_DIR)) {
    fs.mkdirSync(TEST_DIR, { recursive: true });
  }
}

export function cleanupTestDirectories() {
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
// IVF (Indeo Video Format) File Generation
// ============================================================================

export interface IVFHeader {
  signature: string; // "DKIF"
  version: number;
  headerSize: number;
  fourCC: string; // "AV01", "VP90", "VP80"
  width: number;
  height: number;
  timebaseDen: number;
  timebaseNum: number;
  numFrames: number;
}

export function createIVFHeader(config: MediaConfig, fourCC: string = 'AV01'): Buffer {
  const header = Buffer.alloc(32);
  let offset = 0;

  // Signature (4 bytes)
  header.write('DKIF', offset);
  offset += 4;

  // Version (2 bytes) + reserved (2 bytes)
  header.writeUInt16LE(0, offset);
  offset += 2;
  header.writeUInt16LE(0, offset);
  offset += 2;

  // Header size (4 bytes)
  header.writeUInt32LE(32, offset);
  offset += 4;

  // FourCC (4 bytes)
  header.write(fourCC, offset, 4, 'ascii');
  offset += 4;

  // Width (2 bytes)
  header.writeUInt16LE(config.width, offset);
  offset += 2;

  // Height (2 bytes)
  header.writeUInt16LE(config.height, offset);
  offset += 2;

  // Timebase denominator (4 bytes)
  header.writeUInt32LE(config.framerate, offset);
  offset += 4;

  // Timebase numerator (4 bytes)
  header.writeUInt32LE(1, offset);
  offset += 4;

  // Number of frames (4 bytes)
  const numFrames = config.framerate * config.duration;
  header.writeUInt32LE(numFrames, offset);

  return header;
}

export function createIVFFrameHeader(frameSize: number, timestamp: number): Buffer {
  const frameHeader = Buffer.alloc(12);
  frameHeader.writeUInt32LE(frameSize, 0);
  frameHeader.writeBigUInt64LE(BigInt(timestamp), 4);
  return frameHeader;
}

export function generateIVFFile(
  filename: string,
  config: Partial<MediaConfig> = {},
  fourCC: string = 'AV01'
): string {
  const finalConfig = { ...DEFAULT_MEDIA_CONFIG, ...config };
  const outputPath = path.join(TEST_DIR, filename);

  const header = createIVFHeader(finalConfig, fourCC);
  const numFrames = finalConfig.framerate * finalConfig.duration;

  // Create dummy frame data (simple pattern)
  const ySize = finalConfig.width * finalConfig.height;
  const uvSize = ySize / 4;
  const frameSize = ySize + 2 * uvSize;

  const fileData = [header];

  for (let i = 0; i < numFrames; i++) {
    const frameData = Buffer.alloc(frameSize);
    
    // Fill Y plane with gradient
    for (let y = 0; y < finalConfig.height; y++) {
      for (let x = 0; x < finalConfig.width; x++) {
        const value = Math.floor(((x + y + i * 10) % 256));
        frameData[y * finalConfig.width + x] = value;
      }
    }
    
    // Fill UV planes with 128 (neutral)
    for (let i = ySize; i < frameSize; i++) {
      frameData[i] = 128;
    }

    const frameHeader = createIVFFrameHeader(frameSize, i);
    fileData.push(frameHeader, frameData);
  }

  fs.writeFileSync(outputPath, Buffer.concat(fileData));
  return outputPath;
}

// ============================================================================
// Y4M (YUV4MPEG2) File Generation
// ============================================================================

export function createY4MHeader(config: MediaConfig): string {
  return `YUV4MPEG2 W${config.width} H${config.height} F${config.framerate}:1 Ip A1:1 C420mpeg2\n`;
}

export function generateY4MFile(
  filename: string,
  config: Partial<MediaConfig> = {}
): string {
  const finalConfig = { ...DEFAULT_MEDIA_CONFIG, ...config };
  const outputPath = path.join(TEST_DIR, filename);

  const header = createY4MHeader(finalConfig);
  const numFrames = finalConfig.framerate * finalConfig.duration;

  const ySize = finalConfig.width * finalConfig.height;
  const uvSize = ySize / 4;
  const frameSize = ySize + 2 * uvSize;

  const fileData = [Buffer.from(header)];

  for (let i = 0; i < numFrames; i++) {
    const frameData = Buffer.alloc(frameSize);
    
    // Fill Y plane with gradient
    for (let y = 0; y < finalConfig.height; y++) {
      for (let x = 0; x < finalConfig.width; x++) {
        const value = Math.floor(((x + y + i * 10) % 256));
        frameData[y * finalConfig.width + x] = value;
      }
    }
    
    // Fill UV planes with 128 (neutral)
    for (let i = ySize; i < frameSize; i++) {
      frameData[i] = 128;
    }

    fileData.push(Buffer.from('FRAME\n'), frameData);
  }

  fs.writeFileSync(outputPath, Buffer.concat(fileData));
  return outputPath;
}

// ============================================================================
// Matroska/WebM File Generation (Simplified EBML)
// ============================================================================

export function generateMatroskaFile(
  filename: string,
  config: Partial<MediaConfig> = {}
): string {
  const finalConfig = { ...DEFAULT_MEDIA_CONFIG, ...config };
  const outputPath = path.join(TEST_DIR, filename);

  // Simplified EBML header for Matroska/WebM
  const ebmlHeader = Buffer.from([
    0x1a, 0x45, 0xdf, 0xa3, // EBML header
    0x93, // EBML header size
    0x42, 0x86, // EBMLVersion
    0x80, // Version 1
    0x42, 0xf7, // EBMLReadVersion
    0x80,
    0x42, 0xf2, // EBMLMaxIDLength
    0x80,
    0x42, 0xf3, // EBMLMaxSizeLength
    0x42, 0x82, // DocType
    0x84,
    0x77, 0x65, 0x62, 0x6d, // "webm"
  ]);

  // Create dummy frame data
  const ySize = finalConfig.width * finalConfig.height;
  const uvSize = ySize / 4;
  const frameSize = ySize + 2 * uvSize;
  const numFrames = finalConfig.framerate * finalConfig.duration;

  const frameData = Buffer.alloc(frameSize);
  for (let i = 0; i < frameSize; i++) {
    frameData[i] = 128;
  }

  const fileData = [ebmlHeader];

  for (let i = 0; i < numFrames; i++) {
    // SimpleBlock element (0xA3)
    const blockSize = 4 + frameData.length; // track(1) + timestamp(2) + flags(1) + data
    const blockHeader = Buffer.from([0xA3, blockSize & 0x7F, 0x81, 0x00, 0x00, 0x80]);
    fileData.push(blockHeader, frameData);
  }

  fs.writeFileSync(outputPath, Buffer.concat(fileData));
  return outputPath;
}

// ============================================================================
// Media Validation Helpers
// ============================================================================

export function validateIVFFile(filePath: string): boolean {
  if (!fs.existsSync(filePath)) {
    return false;
  }

  const data = fs.readFileSync(filePath);
  
  // Check signature
  if (data.length < 4 || data.toString('ascii', 0, 4) !== 'DKIF') {
    return false;
  }

  // Check header size
  const headerSize = data.readUInt32LE(8);
  if (headerSize !== 32) {
    return false;
  }

  return true;
}

export function validateY4MFile(filePath: string): boolean {
  if (!fs.existsSync(filePath)) {
    return false;
  }

  const data = fs.readFileSync(filePath);
  
  // Check signature
  if (data.length < 9 || data.toString('ascii', 0, 9) !== 'YUV4MPEG2') {
    return false;
  }

  return true;
}

export function validateMatroskaFile(filePath: string): boolean {
  if (!fs.existsSync(filePath)) {
    return false;
  }

  const data = fs.readFileSync(filePath);
  
  // Check EBML signature
  if (data.length < 4 || 
      data[0] !== 0x1a || data[1] !== 0x45 || 
      data[2] !== 0xdf || data[3] !== 0xa3) {
    return false;
  }

  return true;
}

// ============================================================================
// Export All
// ============================================================================

export default {
  // Directory management
  setupTestDirectories,
  cleanupTestDirectories,

  // IVF generation
  createIVFHeader,
  createIVFFrameHeader,
  generateIVFFile,
  validateIVFFile,

  // Y4M generation
  createY4MHeader,
  generateY4MFile,
  validateY4MFile,

  // Matroska generation
  generateMatroskaFile,
  validateMatroskaFile,

  // Configuration
  TEST_DIR,
  DEFAULT_MEDIA_CONFIG,
};
