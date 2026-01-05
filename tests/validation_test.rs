//! Validation tests for media files
//!
//! These tests validate that transcoded media files are valid and playable.

use std::fs;
use std::path::PathBuf;

/// Test directory for media files
fn get_test_dir() -> PathBuf {
  PathBuf::from("examples/test_files")
}

/// Create a minimal valid Y4M file for testing
fn create_test_y4m_file(path: &PathBuf, width: u32, height: u32, num_frames: u32) {
  let mut file = fs::File::create(path).expect("Failed to create test Y4M file");

  // Write Y4M header
  let header = format!("YUV4MPEG2 W{} H{} F30:1 Ip A1:1 C420mpeg2\n", width, height);
  use std::io::Write;
  file
    .write_all(header.as_bytes())
    .expect("Failed to write Y4M header");

  // Write frames
  let y_size = (width * height) as usize;
  let uv_size = y_size / 4;
  let frame_size = y_size + 2 * uv_size;

  for _ in 0..num_frames {
    file
      .write_all(b"FRAME\n")
      .expect("Failed to write FRAME marker");

    // Write Y plane (gradient)
    for y in 0..height {
      for x in 0..width {
        let value = ((x as u32 * 255) / width) as u8;
        file.write_all(&[value]).expect("Failed to write Y pixel");
      }
    }

    // Write U plane (constant)
    file
      .write_all(&vec![128u8; uv_size])
      .expect("Failed to write U plane");

    // Write V plane (constant)
    file
      .write_all(&vec![128u8; uv_size])
      .expect("Failed to write V plane");
  }
}

/// Test Y4M file validation
#[test]
fn test_validate_y4m_file() {
  let test_dir = get_test_dir();
  fs::create_dir_all(&test_dir).expect("Failed to create test directory");

  let test_file = test_dir.join("test_validation_y4m.y4m");
  create_test_y4m_file(&test_file, 320, 240, 10);

  // Validate the file
  let result = rust_av_kit::validation::validate_media_file(&test_file);

  assert!(result.is_valid, "Y4M file should be valid");
  assert!(!result.format.is_empty(), "Format should be detected");
  assert!(result.errors.is_empty(), "Should have no errors");

  // Clean up
  fs::remove_file(&test_file).ok();
}

/// Test empty file validation
#[test]
fn test_validate_empty_file() {
  let test_dir = get_test_dir();
  fs::create_dir_all(&test_dir).expect("Failed to create test directory");

  let test_file = test_dir.join("test_empty.y4m");
  fs::File::create(&test_file).expect("Failed to create empty file");

  let result = rust_av_kit::validation::validate_media_file(&test_file);

  assert!(!result.is_valid, "Empty file should be invalid");
  assert!(!result.errors.is_empty(), "Should have errors");

  // Clean up
  fs::remove_file(&test_file).ok();
}

/// Test non-existent file validation
#[test]
fn test_validate_nonexistent_file() {
  let test_file = PathBuf::from("/nonexistent/test_file.y4m");

  let result = rust_av_kit::validation::validate_media_file(&test_file);

  assert!(!result.is_valid, "Non-existent file should be invalid");
  assert!(!result.errors.is_empty(), "Should have errors");
}

/// Test file size validation
#[test]
fn test_validate_small_file() {
  let test_dir = get_test_dir();
  fs::create_dir_all(&test_dir).expect("Failed to create test directory");

  let test_file = test_dir.join("test_small.y4m");
  fs::write(&test_file, b"YUV4MPEG2 W320 H240\n").expect("Failed to write small file");

  let result = rust_av_kit::validation::validate_media_file(&test_file);

  assert!(!result.is_valid, "File too small should be invalid");
  assert!(!result.errors.is_empty(), "Should have errors");

  // Clean up
  fs::remove_file(&test_file).ok();
}

/// Test FFmpeg availability check
#[test]
fn test_ffmpeg_availability() {
  let available = rust_av_kit::validation::check_ffmpeg_available();
  // This test just checks that the function runs without panicking
  // The result depends on whether FFmpeg is installed
  println!("FFmpeg available: {}", available);
}

/// Test MediaInfo availability check
#[test]
fn test_mediainfo_availability() {
  let available = rust_av_kit::validation::check_mediainfo_available();
  // This test just checks that the function runs without panicking
  // The result depends on whether MediaInfo is installed
  println!("MediaInfo available: {}", available);
}

/// Test Y4M round-trip validation
#[test]
fn test_y4m_roundtrip_validation() {
  let test_dir = get_test_dir();
  fs::create_dir_all(&test_dir).expect("Failed to create test directory");

  // Create test file
  let input_file = test_dir.join("test_roundtrip_input.y4m");
  create_test_y4m_file(&input_file, 320, 240, 5);

  // Validate input
  let input_result = rust_av_kit::validation::validate_media_file(&input_file);
  assert!(input_result.is_valid, "Input Y4M file should be valid");

  // Note: We can't test actual transcoding here without proper codec implementation
  // This test validates the validation infrastructure

  // Clean up
  fs::remove_file(&input_file).ok();
}

/// Test ValidationResult struct
#[test]
fn test_validation_result_struct() {
  let mut result = rust_av_kit::validation::ValidationResult::new();

  assert!(!result.is_valid);
  assert_eq!(result.format, "");
  assert!(result.duration.is_none());
  assert!(result.width.is_none());
  assert!(result.height.is_none());
  assert!(result.codec.is_none());
  assert!(result.frame_count.is_none());
  assert!(result.errors.is_empty());
  assert!(result.warnings.is_empty());

  // Add an error
  result.add_error("Test error".to_string());
  assert!(!result.is_valid);
  assert_eq!(result.errors.len(), 1);

  // Add a warning
  result.add_warning("Test warning".to_string());
  assert_eq!(result.warnings.len(), 1);

  // Finalize
  result.finalize();
  assert!(!result.is_valid); // Still invalid due to error
}

/// Test ValidationResult with valid data
#[test]
fn test_validation_result_valid() {
  let mut result = rust_av_kit::validation::ValidationResult::new();

  result.format = "yuv4mpeg".to_string();
  result.duration = Some(5.0);
  result.width = Some(320);
  result.height = Some(240);
  result.codec = Some("rawvideo".to_string());
  result.frame_count = Some(150);

  result.finalize();

  assert!(result.is_valid);
  assert_eq!(result.format, "yuv4mpeg");
  assert_eq!(result.duration, Some(5.0));
  assert_eq!(result.width, Some(320));
  assert_eq!(result.height, Some(240));
  assert_eq!(result.codec, Some("rawvideo".to_string()));
  assert_eq!(result.frame_count, Some(150));
}

/// Test multiple Y4M files with different resolutions
#[test]
fn test_validate_multiple_resolutions() {
  let test_dir = get_test_dir();
  fs::create_dir_all(&test_dir).expect("Failed to create test directory");

  let resolutions = vec![(160, 120), (320, 240), (640, 480), (1280, 720)];

  for (width, height) in resolutions {
    let test_file = test_dir.join(format!("test_{}x{}.y4m", width, height));
    create_test_y4m_file(&test_file, width, height, 5);

    let result = rust_av_kit::validation::validate_media_file(&test_file);

    assert!(
      result.is_valid,
      "Y4M file at {}x{} should be valid",
      width, height
    );

    // Clean up
    fs::remove_file(&test_file).ok();
  }
}

/// Test validation with warnings
#[test]
fn test_validation_with_warnings() {
  let test_dir = get_test_dir();
  fs::create_dir_all(&test_dir).expect("Failed to create test directory");

  let test_file = test_dir.join("test_warnings.y4m");
  create_test_y4m_file(&test_file, 320, 240, 5);

  let mut result = rust_av_kit::validation::validate_media_file(&test_file);

  // Simulate adding warnings (in real scenario, these might come from FFmpeg)
  result.add_warning("Codec not fully supported".to_string());

  assert!(result.is_valid, "File with warnings should still be valid");
  assert_eq!(result.warnings.len(), 1);

  // Clean up
  fs::remove_file(&test_file).ok();
}
