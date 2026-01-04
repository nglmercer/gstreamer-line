#[cfg(test)]
mod integration_tests {
  extern crate rust_av_kit;

  use std::fs;
  use std::path::PathBuf;

  // Import the library types and functions
  use rust_av_kit::{
    get_supported_codecs, get_supported_formats, get_supported_pixel_formats,
    get_supported_sample_formats, CodecOptions, FilterConfig, FormatInfo, MediaInfo, ProgressData,
    StreamInfo, TranscodeOptions,
  };

  // Helper function to create a test directory
  fn setup_test_dir() -> PathBuf {
    let test_dir = PathBuf::from("temp_frames/test_output");
    fs::create_dir_all(&test_dir).unwrap();
    test_dir
  }

  // Helper function to clean up test directory
  fn cleanup_test_dir(test_dir: PathBuf) {
    if test_dir.exists() {
      fs::remove_dir_all(&test_dir).unwrap();
    }
  }

  #[test]
  fn test_get_supported_formats() {
    let formats = get_supported_formats();
    assert!(!formats.is_empty());
    assert!(formats.contains(&"ivf".to_string()));
    assert!(formats.contains(&"webm".to_string()));
  }

  #[test]
  fn test_get_supported_codecs() {
    let codecs = get_supported_codecs();
    assert!(!codecs.is_empty());
    assert!(codecs.contains(&"av1".to_string()));
    assert!(codecs.contains(&"opus".to_string()));
  }

  #[test]
  fn test_get_supported_pixel_formats() {
    let pixel_formats = get_supported_pixel_formats();
    assert!(!pixel_formats.is_empty());
    assert!(pixel_formats.contains(&"yuv420p".to_string()));
    assert!(pixel_formats.contains(&"rgb24".to_string()));
  }

  #[test]
  fn test_get_supported_sample_formats() {
    let sample_formats = get_supported_sample_formats();
    assert!(!sample_formats.is_empty());
    assert!(sample_formats.contains(&"s16".to_string()));
    assert!(sample_formats.contains(&"f32".to_string()));
  }

  #[test]
  fn test_media_info_structs() {
    // Test that the structs can be created and cloned
    let stream_info = StreamInfo {
      index: 0,
      codec_type: "video".to_string(),
      codec_name: "h264".to_string(),
      bit_rate: Some(1000000),
      width: Some(1920),
      height: Some(1080),
      frame_rate: Some(30.0),
      sample_rate: None,
      channels: None,
      duration: Some(10.0),
    };

    let format_info = FormatInfo {
      name: "mp4".to_string(),
      long_name: "MP4 format".to_string(),
      duration: Some(10.0),
      bit_rate: Some(2000000),
      start_time: Some(0),
      nb_streams: 1,
    };

    let media_info = MediaInfo {
      format: format_info,
      streams: vec![stream_info],
    };

    // Test cloning
    let _media_info_clone = media_info.clone();
  }

  #[test]
  fn test_transcode_options_structs() {
    let codec_options = CodecOptions {
      codec_name: Some("h264".to_string()),
      bit_rate: Some(1000000),
      width: Some(1920),
      height: Some(1080),
      frame_rate: Some(30.0),
      sample_rate: None,
      channels: None,
      gop_size: Some(30),
      max_b_frames: Some(3),
      crf: Some(23),
      preset: Some("medium".to_string()),
      tune: None,
      profile: Some("high".to_string()),
      level: Some(40),
    };

    let filter_config = FilterConfig {
      filter_string: "scale=1280:720".to_string(),
    };

    let transcode_options = TranscodeOptions {
      input_path: "input.mp4".to_string(),
      output_path: "output.mp4".to_string(),
      video_codec: Some(codec_options),
      audio_codec: None,
      video_filter: Some(filter_config),
      audio_filter: None,
      format: Some("mp4".to_string()),
      start_time: Some(0.0),
      duration: Some(10.0),
      seek_to: None,
    };

    // Test cloning
    let _options_clone = transcode_options.clone();
  }

  #[test]
  fn test_progress_data_struct() {
    let progress_data = ProgressData {
      current_time: 5.0,
      total_time: 10.0,
      percentage: 50.0,
      fps: Some(30.0),
      bit_rate: Some(1000000),
      size: 5000000,
    };

    assert_eq!(progress_data.current_time, 5.0);
    assert_eq!(progress_data.total_time, 10.0);
    assert_eq!(progress_data.percentage, 50.0);
    assert_eq!(progress_data.size, 5000000);
  }
}
