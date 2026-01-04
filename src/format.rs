//! Format detection and handling module
//!
//! This module provides functionality for detecting and working with media container formats.

use std::path::Path;

/// Supported media container formats
#[derive(Debug, Clone, PartialEq)]
pub enum MediaFormat {
  /// IVF (Indeo Video Format) - for VP9/AV1 bitstreams
  Ivf,
  /// Matroska/WebM container
  Matroska,
  /// Y4M uncompressed video format
  Y4m,
  /// Unknown format
  Unknown(String),
}

/// Detect media format from file extension
pub fn detect_format(path: &Path) -> MediaFormat {
  match path.extension().and_then(|ext| ext.to_str()) {
    Some("ivf") => MediaFormat::Ivf,
    Some("mkv") | Some("webm") => MediaFormat::Matroska,
    Some("y4m") => MediaFormat::Y4m,
    Some(ext) => MediaFormat::Unknown(ext.to_lowercase()),
    None => MediaFormat::Unknown(String::new()),
  }
}

/// Get format name
pub fn format_name(format: &MediaFormat) -> String {
  match format {
    MediaFormat::Ivf => "ivf".to_string(),
    MediaFormat::Matroska => "matroska".to_string(),
    MediaFormat::Y4m => "y4m".to_string(),
    MediaFormat::Unknown(name) => name.clone(),
  }
}

/// Get format long name/description
pub fn format_long_name(format: &MediaFormat) -> String {
  match format {
    MediaFormat::Ivf => "Indeo Video Format (IVF)".to_string(),
    MediaFormat::Matroska => "Matroska/WebM container".to_string(),
    MediaFormat::Y4m => "YUV4MPEG2 uncompressed video".to_string(),
    MediaFormat::Unknown(name) => format!("Unknown format: {}", name),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_detect_ivf_format() {
    let path = Path::new("test.ivf");
    assert_eq!(detect_format(path), MediaFormat::Ivf);
  }

  #[test]
  fn test_detect_matroska_format() {
    let path = Path::new("test.mkv");
    assert_eq!(detect_format(path), MediaFormat::Matroska);

    let path = Path::new("test.webm");
    assert_eq!(detect_format(path), MediaFormat::Matroska);
  }

  #[test]
  fn test_detect_y4m_format() {
    let path = Path::new("test.y4m");
    assert_eq!(detect_format(path), MediaFormat::Y4m);
  }

  #[test]
  fn test_detect_unknown_format() {
    let path = Path::new("test.mp4");
    assert!(matches!(detect_format(path), MediaFormat::Unknown(_)));
  }

  #[test]
  fn test_format_name() {
    assert_eq!(format_name(&MediaFormat::Ivf), "ivf");
    assert_eq!(format_name(&MediaFormat::Matroska), "matroska");
    assert_eq!(format_name(&MediaFormat::Y4m), "y4m");
  }
}
