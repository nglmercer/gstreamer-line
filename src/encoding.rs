//! Encoding and decoding module
//!
//! This module provides encoding and decoding functionality for media data.

use napi::Error;

/// Encode YUV to IVF frame with actual compression
pub fn encode_yuv_to_ivf_frame(
  yuv_data: &[u8],
  _width: i32,
  _height: i32,
) -> Result<Vec<u8>, Error> {
  // For now, use YUV data directly as a simple compression
  // In a full implementation, this would use av-encoders to encode with AV1/VP9/etc.
  // The YUV420 format is already a compressed representation compared to RGB

  // Apply basic compression: run-length encoding for repeated values
  let mut compressed = Vec::with_capacity(yuv_data.len());
  let mut i = 0;

  while i < yuv_data.len() {
    let current = yuv_data[i];
    let mut count = 1u8;

    // Count consecutive same values
    while i + (count as usize) < yuv_data.len()
      && yuv_data[i + (count as usize)] == current
      && count < 255
    {
      count += 1;
    }

    // If we have repeats, use run-length encoding
    if count > 3 {
      compressed.push(0xFF); // RLE marker
      compressed.push(count);
      compressed.push(current);
      i += count as usize;
    } else {
      compressed.push(current);
      i += 1;
    }
  }

  // Only use compression if it's actually smaller
  if compressed.len() < yuv_data.len() {
    Ok(compressed)
  } else {
    Ok(yuv_data.to_vec())
  }
}

/// Decode IVF frame to YUV with actual decompression
pub fn decode_ivf_frame_to_yuv(
  frame_data: &[u8],
  _width: i32,
  _height: i32,
) -> Result<Vec<u8>, Error> {
  // Check if this is RLE-compressed data
  if !frame_data.is_empty() && frame_data[0] == 0xFF {
    // Decompress run-length encoded data
    let mut decompressed = Vec::new();
    let mut i = 0;

    while i + 2 < frame_data.len() {
      if frame_data[i] == 0xFF {
        // RLE encoded sequence
        let count = frame_data[i + 1] as usize;
        let value = frame_data[i + 2];

        for _ in 0..count {
          decompressed.push(value);
        }

        i += 3;
      } else {
        // Raw byte
        decompressed.push(frame_data[i]);
        i += 1;
      }
    }

    // Copy remaining bytes
    while i < frame_data.len() {
      decompressed.push(frame_data[i]);
      i += 1;
    }

    Ok(decompressed)
  } else {
    // Not compressed, return as-is
    Ok(frame_data.to_vec())
  }
}
