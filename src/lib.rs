// Re-export modular components
pub mod codec;
pub mod codec_detection;
pub mod encoding;
pub mod format;
pub mod format_parsers;
pub mod format_writers;
pub mod media;
pub mod transcoding;
pub mod types;
pub mod validation;
pub mod video_encoding;
pub mod video_filters;

pub use codec::*;
pub use format::*;
pub use media::*;
pub use types::*;

// Explicit re-exports to avoid ambiguity
pub use validation::{validate_file, validate_media_file, ValidationResult};

use codec_detection::{detect_codec_from_data, estimate_duration};
use napi_derive::napi;
use std::path::PathBuf;
use std::sync::Mutex;
use transcoding::*;

// Re-export frame extraction types and functions
pub use transcoding::{
  extract_frames_as_rgba, extract_frames_to_images, extract_frames_with_v_frame,
  save_frames_as_images, FrameData, SaveFramesOptions,
};

// Initialize rust-av on module load
static RUST_AV_INIT: Mutex<bool> = Mutex::new(false);

fn init_rust_av() {
  let mut initialized = RUST_AV_INIT.lock().unwrap();
  if !*initialized {
    // rust-av ecosystem doesn't require explicit initialization
    *initialized = true;
  }
}

/// Get media information from a file
///
/// Uses av-format to read media files and extract metadata including
/// format information and stream details.
#[napi]
pub fn get_media_info(path: String) -> Result<MediaInfo, napi::Error> {
  init_rust_av();

  let path_buf = PathBuf::from(&path);
  if !path_buf.exists() {
    return Err(napi::Error::from_reason(format!(
      "File not found: {}",
      path_buf.display()
    )));
  }

  // Check if file is empty
  let metadata = std::fs::metadata(&path_buf)
    .map_err(|e| napi::Error::from_reason(format!("Failed to read file metadata: {}", e)))?;

  let file_size = metadata.len();
  if file_size == 0 {
    return Err(napi::Error::from_reason("File is empty"));
  }
  if file_size < 32 {
    return Err(napi::Error::from_reason(
      "File is too small to be a valid media file",
    ));
  }

  // Detect format from file
  let detected_format = format::detect_format(&path_buf);

  let format_name = match &detected_format {
    format::MediaFormat::Ivf => "ivf",
    format::MediaFormat::Matroska => "matroska",
    format::MediaFormat::Y4m => "y4m",
    format::MediaFormat::Unknown(name) => name,
  };

  let format_long_name = format::format_long_name(&detected_format);

  // Read file to detect codec and stream information
  let mut buffer = vec![0u8; std::cmp::min(8192, file_size as usize)];
  use std::io::Read;
  let mut file_handle = std::fs::File::open(&path_buf)?;
  let _bytes_read = file_handle.read(&mut buffer)?;

  // Detect codec from file signature
  let (codec_name, codec_type, width, height, frame_rate, sample_rate, channels) =
    detect_codec_from_data(&buffer, &detected_format, &path_buf);

  // Validate that we got meaningful data
  if codec_name.is_empty() && width.is_none() && height.is_none() {
    return Err(napi::Error::from_reason("Invalid or corrupted media file"));
  }

  // Calculate approximate duration based on file size and codec
  let duration = estimate_duration(file_size, &codec_name, width, height, frame_rate);

  // Calculate approximate bit rate
  let bit_rate = if duration > 0.0 {
    Some((file_size as f64 * 8.0 / duration) as i64)
  } else {
    None
  };

  // Create stream info
  let stream_info = if !codec_name.is_empty() {
    vec![StreamInfo {
      index: 0,
      codec_type: codec_type.clone(),
      codec_name: codec_name.clone(),
      bit_rate,
      width,
      height,
      frame_rate,
      sample_rate,
      channels,
      duration: Some(duration),
    }]
  } else {
    vec![]
  };

  Ok(MediaInfo {
    format: FormatInfo {
      name: format_name.to_string(),
      long_name: format_long_name,
      duration: if duration > 0.0 { Some(duration) } else { None },
      bit_rate,
      start_time: Some(0),
      nb_streams: stream_info.len() as i32,
    },
    streams: stream_info,
  })
}

/// Transcode media file
///
/// Performs actual transcoding using av-format, av-data, and v_frame crates.
/// This includes decoding input frames, applying filters, and encoding to output format.
#[napi]
pub fn transcode(options: TranscodeOptions) -> Result<(), napi::Error> {
  init_rust_av();

  let input_path = PathBuf::from(&options.input_path);
  let output_path = PathBuf::from(&options.output_path);

  if !input_path.exists() {
    return Err(napi::Error::from_reason(format!(
      "Input file not found: {}",
      input_path.display()
    )));
  }

  // Detect input and output formats
  let input_format = format::detect_format(&input_path);
  let output_format = format::detect_format(&output_path);

  // Read input file
  let input_data = std::fs::read(&input_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to read input file: {}", e)))?;

  // Process based on format combination
  match (&input_format, &output_format) {
    (format::MediaFormat::Ivf, format::MediaFormat::Matroska) => {
      transcode_ivf_to_matroska(&input_data, &output_path, &options)?;
    }
    (format::MediaFormat::Matroska, format::MediaFormat::Ivf) => {
      transcode_matroska_to_ivf(&input_data, &output_path, &options)?;
    }
    (format::MediaFormat::Y4m, format::MediaFormat::Ivf) => {
      transcode_y4m_to_ivf(&input_data, &output_path, &options)?;
    }
    (format::MediaFormat::Ivf, format::MediaFormat::Y4m) => {
      transcode_ivf_to_y4m(&input_data, &output_path, &options)?;
    }
    (format::MediaFormat::Y4m, format::MediaFormat::Matroska) => {
      transcode_y4m_to_matroska(&input_data, &output_path, &options)?;
    }
    (format::MediaFormat::Matroska, format::MediaFormat::Y4m) => {
      transcode_matroska_to_y4m(&input_data, &output_path, &options)?;
    }
    _ => {
      return Err(napi::Error::from_reason(format!(
        "Unsupported transcoding from {:?} to {:?}",
        input_format, output_format
      )));
    }
  }

  Ok(())
}

/// Get supported formats
///
/// Returns a list of container formats supported by Rust-AV ecosystem.
#[napi]
pub fn get_supported_formats() -> Vec<String> {
  let processor = create_processor();
  processor.supported_formats()
}

/// Get supported codecs
///
/// Returns a list of codecs supported by Rust-AV ecosystem.
#[napi]
pub fn get_supported_codecs() -> Vec<String> {
  let processor = create_processor();
  processor.supported_codecs()
}

/// Get supported pixel formats
///
/// Returns a list of pixel formats supported by v_frame.
#[napi]
pub fn get_supported_pixel_formats() -> Vec<String> {
  vec![
    "yuv420p".to_string(),
    "yuv422p".to_string(),
    "yuv444p".to_string(),
    "rgb24".to_string(),
    "bgr24".to_string(),
    "rgba".to_string(),
  ]
}

/// Get supported sample formats
///
/// Returns a list of audio sample formats supported by av-data.
#[napi]
pub fn get_supported_sample_formats() -> Vec<String> {
  vec![
    "u8".to_string(),
    "s16".to_string(),
    "s32".to_string(),
    "f32".to_string(),
  ]
}

/// Transform media file from one format to another
///
/// Converts a media file from its current format to a target format.
/// Uses actual transcoding implementation with proper format handling.
#[napi]
pub fn transform_format(input_path: String, output_path: String) -> Result<(), napi::Error> {
  init_rust_av();

  let input_buf = PathBuf::from(&input_path);
  let output_buf = PathBuf::from(&output_path);

  if !input_buf.exists() {
    return Err(napi::Error::from_reason(format!(
      "Input file not found: {}",
      input_buf.display()
    )));
  }

  let input_format = format::detect_format(&input_buf);
  let output_format = format::detect_format(&output_buf);

  // Read input file
  let input_data = std::fs::read(&input_buf)
    .map_err(|e| napi::Error::from_reason(format!("Failed to read input file: {}", e)))?;

  // Process based on format combination using real transcoding functions
  match (&input_format, &output_format) {
    (format::MediaFormat::Ivf, format::MediaFormat::Matroska) => {
      transcode_ivf_to_matroska(
        &input_data,
        &output_buf,
        &TranscodeOptions {
          input_path,
          output_path,
          video_codec: None,
          audio_codec: None,
          video_filter: None,
          audio_filter: None,
          format: None,
          start_time: None,
          duration: None,
          seek_to: None,
        },
      )?;
    }
    (format::MediaFormat::Matroska, format::MediaFormat::Ivf) => {
      transcode_matroska_to_ivf(
        &input_data,
        &output_buf,
        &TranscodeOptions {
          input_path,
          output_path,
          video_codec: None,
          audio_codec: None,
          video_filter: None,
          audio_filter: None,
          format: None,
          start_time: None,
          duration: None,
          seek_to: None,
        },
      )?;
    }
    (format::MediaFormat::Y4m, format::MediaFormat::Ivf) => {
      transcode_y4m_to_ivf(
        &input_data,
        &output_buf,
        &TranscodeOptions {
          input_path,
          output_path,
          video_codec: None,
          audio_codec: None,
          video_filter: None,
          audio_filter: None,
          format: None,
          start_time: None,
          duration: None,
          seek_to: None,
        },
      )?;
    }
    (format::MediaFormat::Ivf, format::MediaFormat::Y4m) => {
      transcode_ivf_to_y4m(
        &input_data,
        &output_buf,
        &TranscodeOptions {
          input_path,
          output_path,
          video_codec: None,
          audio_codec: None,
          video_filter: None,
          audio_filter: None,
          format: None,
          start_time: None,
          duration: None,
          seek_to: None,
        },
      )?;
    }
    (format::MediaFormat::Y4m, format::MediaFormat::Matroska) => {
      transcode_y4m_to_matroska(
        &input_data,
        &output_buf,
        &TranscodeOptions {
          input_path,
          output_path,
          video_codec: None,
          audio_codec: None,
          video_filter: None,
          audio_filter: None,
          format: None,
          start_time: None,
          duration: None,
          seek_to: None,
        },
      )?;
    }
    (format::MediaFormat::Matroska, format::MediaFormat::Y4m) => {
      transcode_matroska_to_y4m(
        &input_data,
        &output_buf,
        &TranscodeOptions {
          input_path,
          output_path,
          video_codec: None,
          audio_codec: None,
          video_filter: None,
          audio_filter: None,
          format: None,
          start_time: None,
          duration: None,
          seek_to: None,
        },
      )?;
    }
    _ => {
      return Err(napi::Error::from_reason(format!(
        "Unsupported format conversion from {:?} to {:?}",
        input_format, output_format
      )));
    }
  }

  Ok(())
}
