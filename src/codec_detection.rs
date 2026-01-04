//! Codec detection module
//!
//! This module provides functionality for detecting codecs from file data.

use crate::format::MediaFormat;
use std::path::Path;

/// Type alias for codec detection result to reduce type complexity
pub type CodecDetectionResult = (
  String,
  String,
  Option<i32>,
  Option<i32>,
  Option<f64>,
  Option<i32>,
  Option<i32>,
);

/// Detect codec from file data and format
pub fn detect_codec_from_data(
  data: &[u8],
  format: &MediaFormat,
  path: &Path,
) -> CodecDetectionResult {
  match format {
    MediaFormat::Ivf => detect_ivf_codec(data),
    MediaFormat::Matroska => detect_matroska_codec(data, path),
    MediaFormat::Y4m => detect_y4m_codec(data),
    MediaFormat::Unknown(_) => (
      String::new(),
      String::new(),
      None,
      None,
      None,
      None,
      None,
    ),
  }
}

/// Detect codec from IVF file data
fn detect_ivf_codec(data: &[u8]) -> CodecDetectionResult {
  // IVF header: DKIF + version + header size + fourcc
  if data.len() >= 32 && &data[0..4] == b"DKIF" {
    let fourcc = std::str::from_utf8(&data[16..20]).unwrap_or("unknown");
    let width = u16::from_le_bytes([data[24], data[25]]) as i32;
    let height = u16::from_le_bytes([data[26], data[27]]) as i32;
    let timebase_den = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
    let frame_rate = if timebase_den > 0 {
      Some(30.0) // Default frame rate for IVF
    } else {
      None
    };

    let codec_name = match fourcc {
      "AV01" => "av1",
      "VP90" => "vp9",
      "VP80" => "vp8",
      _ => fourcc,
    };

    (
      codec_name.to_string(),
      "video".to_string(),
      Some(width),
      Some(height),
      frame_rate,
      None,
      None,
    )
  } else {
    (String::new(), String::new(), None, None, None, None, None)
  }
}

/// Detect codec from Matroska file data
fn detect_matroska_codec(data: &[u8], path: &Path) -> CodecDetectionResult {
  // Matroska/WebM - detect from file signature
  if data.len() >= 4 && &data[0..4] == b"\x1a\x45\xdf\xa3" {
    // Try to detect codec from file extension or content
    let ext = path
      .extension()
      .and_then(|e: &std::ffi::OsStr| e.to_str())
      .unwrap_or("");
    let (codec_name, codec_type) = match ext {
      "webm" => ("vp9", "video"),
      "mkv" => ("h264", "video"),
      _ => ("unknown", "unknown"),
    };
    (
      codec_name.to_string(),
      codec_type.to_string(),
      None,
      None,
      None,
      None,
      None,
    )
  } else {
    (String::new(), String::new(), None, None, None, None, None)
  }
}

/// Detect codec from Y4M file data
fn detect_y4m_codec(data: &[u8]) -> CodecDetectionResult {
  // Y4M header parsing
  if let Some(header_end) = data.iter().position(|&b| b == b'\n') {
    let header = std::str::from_utf8(&data[..header_end]).unwrap_or("");

    // Parse Y4M header
    let mut width = None;
    let mut height = None;
    let mut frame_rate = None;

    for part in header.split_whitespace() {
      if let Some(rest) = part.strip_prefix("W") {
        width = rest.parse::<i32>().ok();
      } else if let Some(rest) = part.strip_prefix("H") {
        height = rest.parse::<i32>().ok();
      } else if let Some(rest) = part.strip_prefix("F") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
          if let (Ok(num), Ok(den)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            frame_rate = Some(num / den);
          }
        }
      }
    }

    (
      "raw".to_string(),
      "video".to_string(),
      width,
      height,
      frame_rate,
      None,
      None,
    )
  } else {
    (String::new(), String::new(), None, None, None, None, None)
  }
}

/// Estimate duration based on file size and codec
pub fn estimate_duration(
  file_size: u64,
  codec_name: &str,
  width: Option<i32>,
  height: Option<i32>,
  _frame_rate: Option<f64>,
) -> f64 {
  // Rough estimation based on typical bitrates
  let pixels = width.unwrap_or(640) * height.unwrap_or(480);

  let bitrate = match codec_name {
    "av1" => pixels as f64 * 0.1,  // ~0.1 bpp for AV1
    "vp9" => pixels as f64 * 0.15, // ~0.15 bpp for VP9
    "vp8" => pixels as f64 * 0.2,  // ~0.2 bpp for VP8
    "h264" => pixels as f64 * 0.2, // ~0.2 bpp for H.264
    "h265" => pixels as f64 * 0.1, // ~0.1 bpp for H.265
    "raw" => pixels as f64 * 1.5,  // 1.5 bytes per pixel for YUV420
    _ => pixels as f64 * 0.2,
  };

  if bitrate > 0.0 {
    (file_size as f64 * 8.0) / bitrate
  } else {
    0.0
  }
}
