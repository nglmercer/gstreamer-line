//! Validation module for media files
//!
//! This module provides utilities to validate media files and verify
//! that transcoding operations produce valid output.

use napi_derive::napi;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Validation result for a media file
#[napi(object)]
pub struct ValidationResult {
  pub is_valid: bool,
  pub format: String,
  pub duration: Option<f64>,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub codec: Option<String>,
  pub frame_count: Option<i32>,
  pub errors: Vec<String>,
  pub warnings: Vec<String>,
}

impl ValidationResult {
  /// Create a new validation result
  pub fn new() -> Self {
    Self {
      is_valid: false,
      format: String::new(),
      duration: None,
      width: None,
      height: None,
      codec: None,
      frame_count: None,
      errors: Vec::new(),
      warnings: Vec::new(),
    }
  }

  /// Add an error
  pub fn add_error(&mut self, error: String) {
    self.errors.push(error);
    self.is_valid = false;
  }

  /// Add a warning
  pub fn add_warning(&mut self, warning: String) {
    self.warnings.push(warning);
  }

  /// Mark as valid if no errors
  pub fn finalize(&mut self) {
    self.is_valid = self.errors.is_empty();
  }
}

impl Default for ValidationResult {
  fn default() -> Self {
    Self::new()
  }
}

/// Check if FFmpeg is available
pub fn check_ffmpeg_available() -> bool {
  Command::new("ffmpeg")
    .arg("-version")
    .output()
    .map(|output| output.status.success())
    .unwrap_or(false)
}

/// Check if MediaInfo is available
pub fn check_mediainfo_available() -> bool {
  Command::new("mediainfo")
    .arg("--version")
    .output()
    .map(|output| output.status.success())
    .unwrap_or(false)
}

/// Validate a media file using FFmpeg
pub fn validate_with_ffmpeg(file_path: &PathBuf) -> ValidationResult {
  let mut result = ValidationResult::new();

  if !file_path.exists() {
    result.add_error(format!("File does not exist: {}", file_path.display()));
    return result;
  }

  if !check_ffmpeg_available() {
    result.add_warning("FFmpeg not available for validation".to_string());
    return result;
  }

  // Run FFmpeg to probe the file
  let output = Command::new("ffprobe")
    .arg("-v")
    .arg("error")
    .arg("-show_format")
    .arg("-show_streams")
    .arg("-of")
    .arg("json")
    .arg(file_path)
    .output();

  match output {
    Ok(output) => {
      if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        result.add_error(format!("FFprobe failed: {}", stderr));
        return result;
      }

      let stdout = String::from_utf8_lossy(&output.stdout);

      // Parse JSON output
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        // Extract format info
        if let Some(format) = json.get("format") {
          if let Some(format_name) = format.get("format_name") {
            result.format = format_name.as_str().unwrap_or("unknown").to_string();
          }
          if let Some(duration) = format.get("duration") {
            result.duration = duration.as_str().and_then(|s| s.parse().ok());
          }
        }

        // Extract stream info (video stream)
        if let Some(streams) = json.get("streams") {
          if let Some(streams_array) = streams.as_array() {
            for stream in streams_array {
              if let Some(codec_type) = stream.get("codec_type") {
                if codec_type.as_str() == Some("video") {
                  if let Some(codec_name) = stream.get("codec_name") {
                    result.codec = codec_name.as_str().map(|s| s.to_string());
                  }
                  if let Some(width) = stream.get("width") {
                    result.width = width.as_i64().map(|w| w as i32);
                  }
                  if let Some(height) = stream.get("height") {
                    result.height = height.as_i64().map(|h| h as i32);
                  }
                  if let Some(nb_frames) = stream.get("nb_frames") {
                    result.frame_count = nb_frames.as_str().and_then(|s| s.parse().ok());
                  }
                  break;
                }
              }
            }
          }
        }

        result.is_valid = true;
      } else {
        result.add_error("Failed to parse FFprobe JSON output".to_string());
      }
    }
    Err(e) => {
      result.add_error(format!("Failed to run FFprobe: {}", e));
    }
  }

  result
}

/// Validate a media file using MediaInfo
pub fn validate_with_mediainfo(file_path: &PathBuf) -> ValidationResult {
  let mut result = ValidationResult::new();

  if !file_path.exists() {
    result.add_error(format!("File does not exist: {}", file_path.display()));
    return result;
  }

  if !check_mediainfo_available() {
    result.add_warning("MediaInfo not available for validation".to_string());
    return result;
  }

  // Run MediaInfo
  let output = Command::new("mediainfo")
    .arg("--Output=JSON")
    .arg(file_path)
    .output();

  match output {
    Ok(output) => {
      if !output.status.success() {
        result.add_error("MediaInfo failed".to_string());
        return result;
      }

      let stdout = String::from_utf8_lossy(&output.stdout);

      // Simple validation - check if JSON is valid
      if serde_json::from_str::<serde_json::Value>(&stdout).is_ok() {
        result.is_valid = true;
      } else {
        result.add_error("Invalid MediaInfo JSON output".to_string());
      }
    }
    Err(e) => {
      result.add_error(format!("Failed to run MediaInfo: {}", e));
    }
  }

  result
}

/// Comprehensive validation of a media file
pub fn validate_media_file(file_path: &PathBuf) -> ValidationResult {
  let mut result = ValidationResult::new();

  // Basic file checks
  if !file_path.exists() {
    result.add_error(format!("File does not exist: {}", file_path.display()));
    return result;
  }

  let metadata = match std::fs::metadata(file_path) {
    Ok(m) => m,
    Err(e) => {
      result.add_error(format!("Failed to read file metadata: {}", e));
      return result;
    }
  };

  if metadata.len() == 0 {
    result.add_error("File is empty".to_string());
    return result;
  }

  if metadata.len() < 32 {
    result.add_error("File is too small to be a valid media file".to_string());
    return result;
  }

  // Try FFmpeg validation first
  if check_ffmpeg_available() {
    result = validate_with_ffmpeg(file_path);
  } else if check_mediainfo_available() {
    result = validate_with_mediainfo(file_path);
  } else {
    result.add_warning("No validation tools available (FFmpeg or MediaInfo)".to_string());
    // Basic validation - just check file size
    result.is_valid = true;
  }

  result
}

/// Compare two media files for basic similarity
pub fn compare_media_files(file1: &PathBuf, file2: &PathBuf) -> Result<String, String> {
  let result1 = validate_media_file(file1);
  let result2 = validate_media_file(file2);

  if !result1.is_valid {
    return Err(format!("First file is invalid: {:?}", result1.errors));
  }
  if !result2.is_valid {
    return Err(format!("Second file is invalid: {:?}", result2.errors));
  }

  let mut comparison = String::new();

  // Compare format
  if result1.format != result2.format {
    comparison.push_str(&format!(
      "Format differs: {} vs {}\n",
      result1.format, result2.format
    ));
  }

  // Compare dimensions
  if let (Some(w1), Some(h1), Some(w2), Some(h2)) =
    (result1.width, result1.height, result2.width, result2.height)
  {
    if w1 != w2 || h1 != h2 {
      comparison.push_str(&format!(
        "Dimensions differ: {}x{} vs {}x{}\n",
        w1, h1, w2, h2
      ));
    }
  }

  // Compare duration
  if let (Some(d1), Some(d2)) = (result1.duration, result2.duration) {
    let diff = (d1 - d2).abs();
    if diff > 0.1 {
      comparison.push_str(&format!(
        "Duration differs by {:.2}s: {:.2}s vs {:.2}s\n",
        diff, d1, d2
      ));
    }
  }

  // Compare frame count
  if let (Some(f1), Some(f2)) = (result1.frame_count, result2.frame_count) {
    if f1 != f2 {
      comparison.push_str(&format!("Frame count differs: {} vs {}\n", f1, f2));
    }
  }

  if comparison.is_empty() {
    Ok("Files appear to be similar".to_string())
  } else {
    Ok(comparison)
  }
}

/// Print validation result to console
pub fn print_validation_result(result: &ValidationResult, file_path: &Path) {
  println!("\n=== Validation Result for {} ===", file_path.display());
  println!("Valid: {}", result.is_valid);
  println!("Format: {}", result.format);

  if let Some(duration) = result.duration {
    println!("Duration: {:.2}s", duration);
  }

  if let (Some(width), Some(height)) = (result.width, result.height) {
    println!("Dimensions: {}x{}", width, height);
  }

  if let Some(codec) = &result.codec {
    println!("Codec: {}", codec);
  }

  if let Some(frame_count) = result.frame_count {
    println!("Frame Count: {}", frame_count);
  }

  if !result.warnings.is_empty() {
    println!("\nWarnings:");
    for warning in &result.warnings {
      println!("  - {}", warning);
    }
  }

  if !result.errors.is_empty() {
    println!("\nErrors:");
    for error in &result.errors {
      println!("  - {}", error);
    }
  }
  println!();
}

/// Validate a media file and return validation result
#[napi]
pub fn validate_file(file_path: String) -> Result<ValidationResult, napi::Error> {
  let path = PathBuf::from(file_path);
  Ok(validate_media_file(&path))
}
