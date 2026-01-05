//! Format parsers module
//!
//! This module provides functionality for parsing media files in various formats.

use napi::Error;

/// Parse Y4M header
pub fn parse_y4m_header(header: &str) -> Result<(i32, i32, f64), Error> {
  let mut width = 640;
  let mut height = 480;
  let mut frame_rate = 30.0;

  for part in header.split_whitespace() {
    if let Some(rest) = part.strip_prefix("W") {
      width = rest
        .parse::<i32>()
        .map_err(|e| Error::from_reason(format!("Invalid width: {}", e)))?;
    } else if let Some(rest) = part.strip_prefix("H") {
      height = rest
        .parse::<i32>()
        .map_err(|e| Error::from_reason(format!("Invalid height: {}", e)))?;
    } else if let Some(rest) = part.strip_prefix("F") {
      let parts: Vec<&str> = rest.split(':').collect();
      if parts.len() == 2 {
        let num: f64 = parts[0]
          .parse()
          .map_err(|e| Error::from_reason(format!("Invalid frame rate numerator: {}", e)))?;
        let den: f64 = parts[1]
          .parse()
          .map_err(|e| Error::from_reason(format!("Invalid frame rate denominator: {}", e)))?;
        frame_rate = num / den;
      }
    }
  }

  Ok((width, height, frame_rate))
}

/// Parse Matroska frames (simplified)
pub fn parse_matroska_frames(data: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
  let mut frames = Vec::new();

  // Skip EBML header
  let mut offset = if data.len() > 4 && &data[0..4] == b"\x1a\x45\xdf\xa3" {
    4
  } else {
    0
  };

  // Simple parsing - look for frame data patterns
  while offset < data.len() {
    // Look for SimpleBlock element (0xA3)
    if data[offset] == 0xA3 {
      offset += 1;

      // Read size
      let size = if offset < data.len() {
        let first_byte = data[offset];
        if first_byte < 0x7F {
          offset += 1;
          first_byte as usize
        } else {
          // Multi-byte size (simplified)
          offset += 2;
          ((first_byte & 0x7F) as usize) << 8
        }
      } else {
        break;
      };

      // Skip track number and timestamp (simplified)
      offset += 4;

      // Read frame data
      let frame_size = size.saturating_sub(4);
      if offset + frame_size <= data.len() {
        frames.push(data[offset..offset + frame_size].to_vec());
        offset += frame_size;
      } else {
        break;
      }
    } else {
      offset += 1;
    }
  }

  Ok(frames)
}
