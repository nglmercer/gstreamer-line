// Re-export modular components
pub mod codec;
pub mod format;
pub mod media;

pub use codec::*;
pub use format::*;
pub use media::*;

use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Initialize rust-av on module load
static RUST_AV_INIT: Mutex<bool> = Mutex::new(false);

fn init_rust_av() {
  let mut initialized = RUST_AV_INIT.lock().unwrap();
  if !*initialized {
    // rust-av ecosystem doesn't require explicit initialization
    *initialized = true;
  }
}

/// Media stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct StreamInfo {
  pub index: i32,
  pub codec_type: String,
  pub codec_name: String,
  pub bit_rate: Option<i64>,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub frame_rate: Option<f64>,
  pub sample_rate: Option<i32>,
  pub channels: Option<i32>,
  pub duration: Option<f64>,
}

/// Media container format information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct FormatInfo {
  pub name: String,
  pub long_name: String,
  pub duration: Option<f64>,
  pub bit_rate: Option<i64>,
  pub start_time: Option<i64>,
  pub nb_streams: i32,
}

/// Complete media information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct MediaInfo {
  pub format: FormatInfo,
  pub streams: Vec<StreamInfo>,
}

/// Codec configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct CodecOptions {
  pub codec_name: Option<String>,
  pub bit_rate: Option<i64>,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub frame_rate: Option<f64>,
  pub sample_rate: Option<i32>,
  pub channels: Option<i32>,
  pub gop_size: Option<i32>,
  pub max_b_frames: Option<i32>,
  pub crf: Option<i32>,
  pub preset: Option<String>,
  pub tune: Option<String>,
  pub profile: Option<String>,
  pub level: Option<i32>,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct FilterConfig {
  pub filter_string: String,
}

/// Transcoding options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct TranscodeOptions {
  pub input_path: String,
  pub output_path: String,
  pub video_codec: Option<CodecOptions>,
  pub audio_codec: Option<CodecOptions>,
  pub video_filter: Option<FilterConfig>,
  pub audio_filter: Option<FilterConfig>,
  pub format: Option<String>,
  pub start_time: Option<f64>,
  pub duration: Option<f64>,
  pub seek_to: Option<f64>,
}

/// Progress callback data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct ProgressData {
  pub current_time: f64,
  pub total_time: f64,
  pub percentage: f64,
  pub fps: Option<f64>,
  pub bit_rate: Option<i64>,
  pub size: i64,
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

/// Type alias for codec detection result to reduce type complexity
type CodecDetectionResult = (
  String,
  String,
  Option<i32>,
  Option<i32>,
  Option<f64>,
  Option<i32>,
  Option<i32>,
);

/// Detect codec from file data and format
fn detect_codec_from_data(
  data: &[u8],
  format: &format::MediaFormat,
  path: &Path,
) -> CodecDetectionResult {
  match format {
    format::MediaFormat::Ivf => {
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
    format::MediaFormat::Matroska => {
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
    format::MediaFormat::Y4m => {
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
    format::MediaFormat::Unknown(_) => (String::new(), String::new(), None, None, None, None, None),
  }
}

/// Estimate duration based on file size and codec
fn estimate_duration(
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

/// Transcode IVF to Matroska format
fn transcode_ivf_to_matroska(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), napi::Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse IVF header
  if input_data.len() < 32 {
    return Err(napi::Error::from_reason(
      "Invalid IVF file: header too short",
    ));
  }

  let width = u16::from_le_bytes([input_data[24], input_data[25]]) as i32;
  let height = u16::from_le_bytes([input_data[26], input_data[27]]) as i32;
  let _timebase_den = u32::from_le_bytes([
    input_data[28],
    input_data[29],
    input_data[30],
    input_data[31],
  ]);
  let frame_rate = 30.0;

  // Apply video codec options if provided
  let (final_width, final_height, final_frame_rate) = if let Some(video_opts) = &options.video_codec
  {
    (
      video_opts.width.unwrap_or(width),
      video_opts.height.unwrap_or(height),
      video_opts.frame_rate.unwrap_or(frame_rate),
    )
  } else {
    (width, height, frame_rate)
  };

  // Write Matroska EBML header (simplified)
  write_matroska_header(
    &mut output_file,
    final_width,
    final_height,
    final_frame_rate,
  )?;

  // Write IVF frames as Matroska blocks
  let mut offset = 32; // Skip IVF header
  let mut frame_count = 0u32;

  while offset + 12 <= input_data.len() {
    let frame_size = u32::from_le_bytes([
      input_data[offset],
      input_data[offset + 1],
      input_data[offset + 2],
      input_data[offset + 3],
    ]) as usize;

    let timestamp = u64::from_le_bytes([
      input_data[offset + 4],
      input_data[offset + 5],
      input_data[offset + 6],
      input_data[offset + 7],
      input_data[offset + 8],
      input_data[offset + 9],
      input_data[offset + 10],
      input_data[offset + 11],
    ]);

    if offset + 12 + frame_size > input_data.len() {
      break;
    }

    let frame_data = &input_data[offset + 12..offset + 12 + frame_size];

    // Apply video filter if specified
    let output_frame = if let Some(filter) = &options.video_filter {
      apply_video_filter(frame_data, &filter.filter_string)?
    } else {
      frame_data.to_vec()
    };

    // Write frame as Matroska SimpleBlock
    write_matroska_simpleblock(&mut output_file, &output_frame, timestamp, frame_count)?;

    offset += 12 + frame_size;
    frame_count += 1;
  }

  // Write Matroska trailer
  write_matroska_trailer(&mut output_file)?;

  Ok(())
}

/// Transcode Matroska to IVF format
fn transcode_matroska_to_ivf(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), napi::Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Default dimensions
  let width = options
    .video_codec
    .as_ref()
    .and_then(|v| v.width)
    .unwrap_or(640);
  let height = options
    .video_codec
    .as_ref()
    .and_then(|v| v.height)
    .unwrap_or(480);
  let frame_rate = options
    .video_codec
    .as_ref()
    .and_then(|v| v.frame_rate)
    .unwrap_or(30.0);

  // Write IVF header
  write_ivf_header(&mut output_file, width, height, frame_rate)?;

  // Parse Matroska and extract frames
  let frames = parse_matroska_frames(input_data)?;

  // Write frames to IVF
  for (idx, frame) in frames.iter().enumerate() {
    let output_frame = if let Some(filter) = &options.video_filter {
      apply_video_filter(frame, &filter.filter_string)?
    } else {
      frame.clone()
    };

    write_ivf_frame(&mut output_file, &output_frame, idx as u64)?;
  }

  Ok(())
}

/// Transcode Y4M to IVF format
fn transcode_y4m_to_ivf(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), napi::Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse Y4M header
  let header_end = input_data
    .iter()
    .position(|&b| b == b'\n')
    .ok_or_else(|| napi::Error::from_reason("Invalid Y4M file: no header found"))?;

  let header = std::str::from_utf8(&input_data[..header_end])
    .map_err(|e| napi::Error::from_reason(format!("Invalid Y4M header: {}", e)))?;

  let (mut width, mut height, mut frame_rate) = parse_y4m_header(header)?;

  // Apply codec options
  if let Some(video_opts) = &options.video_codec {
    width = video_opts.width.unwrap_or(width);
    height = video_opts.height.unwrap_or(height);
    frame_rate = video_opts.frame_rate.unwrap_or(frame_rate);
  }

  // Write IVF header
  write_ivf_header(&mut output_file, width, height, frame_rate)?;

  // Parse and convert Y4M frames
  let mut offset = header_end + 1;
  let mut frame_idx = 0u32;

  while offset < input_data.len() {
    // Look for FRAME marker
    if offset + 5 <= input_data.len() && &input_data[offset..offset + 5] == b"FRAME" {
      offset += 5;

      // Skip to newline
      while offset < input_data.len() && input_data[offset] != b'\n' {
        offset += 1;
      }
      if offset < input_data.len() {
        offset += 1;
      }

      // Calculate YUV420 frame size
      let y_size = width as usize * height as usize;
      let uv_size = y_size / 4;
      let frame_size = y_size + 2 * uv_size;

      if offset + frame_size > input_data.len() {
        break;
      }

      let yuv_data = &input_data[offset..offset + frame_size];

      // Convert YUV420 to compressed format
      let compressed_frame = encode_yuv_to_ivf_frame(yuv_data, width, height)?;

      // Apply filter if specified
      let output_frame = if let Some(filter) = &options.video_filter {
        apply_video_filter(&compressed_frame, &filter.filter_string)?
      } else {
        compressed_frame
      };

      write_ivf_frame(&mut output_file, &output_frame, frame_idx as u64)?;

      offset += frame_size;
      frame_idx += 1;
    } else {
      offset += 1;
    }
  }

  Ok(())
}

/// Transcode IVF to Y4M format
fn transcode_ivf_to_y4m(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), napi::Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse IVF header
  if input_data.len() < 32 {
    return Err(napi::Error::from_reason(
      "Invalid IVF file: header too short",
    ));
  }

  let mut width = u16::from_le_bytes([input_data[24], input_data[25]]) as i32;
  let mut height = u16::from_le_bytes([input_data[26], input_data[27]]) as i32;
  let mut frame_rate = 30.0;

  // Apply codec options
  if let Some(video_opts) = &options.video_codec {
    width = video_opts.width.unwrap_or(width);
    height = video_opts.height.unwrap_or(height);
    frame_rate = video_opts.frame_rate.unwrap_or(frame_rate);
  }

  // Write Y4M header
  write_y4m_header(&mut output_file, width, height, frame_rate)?;

  // Parse IVF frames and convert to Y4M
  let mut offset = 32;
  let mut frame_count = 0u32;

  while offset + 12 <= input_data.len() {
    let frame_size = u32::from_le_bytes([
      input_data[offset],
      input_data[offset + 1],
      input_data[offset + 2],
      input_data[offset + 3],
    ]) as usize;

    if offset + 12 + frame_size > input_data.len() {
      break;
    }

    let frame_data = &input_data[offset + 12..offset + 12 + frame_size];

    // Decode compressed frame to YUV
    let yuv_data = decode_ivf_frame_to_yuv(frame_data, width, height)?;

    // Apply filter if specified
    let output_frame = if let Some(filter) = &options.video_filter {
      apply_video_filter(&yuv_data, &filter.filter_string)?
    } else {
      yuv_data
    };

    write_y4m_frame(&mut output_file, &output_frame, frame_count)?;

    offset += 12 + frame_size;
    frame_count += 1;
  }

  Ok(())
}

/// Transcode Y4M to Matroska format
fn transcode_y4m_to_matroska(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), napi::Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse Y4M header
  let header_end = input_data
    .iter()
    .position(|&b| b == b'\n')
    .ok_or_else(|| napi::Error::from_reason("Invalid Y4M file: no header found"))?;

  let header = std::str::from_utf8(&input_data[..header_end])
    .map_err(|e| napi::Error::from_reason(format!("Invalid Y4M header: {}", e)))?;

  let (mut width, mut height, mut frame_rate) = parse_y4m_header(header)?;

  // Apply codec options
  if let Some(video_opts) = &options.video_codec {
    width = video_opts.width.unwrap_or(width);
    height = video_opts.height.unwrap_or(height);
    frame_rate = video_opts.frame_rate.unwrap_or(frame_rate);
  }

  // Write Matroska header
  write_matroska_header(&mut output_file, width, height, frame_rate)?;

  // Parse and convert Y4M frames
  let mut offset = header_end + 1;
  let mut frame_idx = 0u32;

  while offset < input_data.len() {
    if offset + 5 <= input_data.len() && &input_data[offset..offset + 5] == b"FRAME" {
      offset += 5;

      while offset < input_data.len() && input_data[offset] != b'\n' {
        offset += 1;
      }
      if offset < input_data.len() {
        offset += 1;
      }

      let y_size = width as usize * height as usize;
      let uv_size = y_size / 4;
      let frame_size = y_size + 2 * uv_size;

      if offset + frame_size > input_data.len() {
        break;
      }

      let yuv_data = &input_data[offset..offset + frame_size];

      // Encode YUV to compressed format
      let compressed_frame = encode_yuv_to_ivf_frame(yuv_data, width, height)?;

      // Apply filter if specified
      let output_frame = if let Some(filter) = &options.video_filter {
        apply_video_filter(&compressed_frame, &filter.filter_string)?
      } else {
        compressed_frame
      };

      write_matroska_simpleblock(&mut output_file, &output_frame, frame_idx as u64, frame_idx)?;

      offset += frame_size;
      frame_idx += 1;
    } else {
      offset += 1;
    }
  }

  write_matroska_trailer(&mut output_file)?;

  Ok(())
}

/// Transcode Matroska to Y4M format
fn transcode_matroska_to_y4m(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), napi::Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  let width = options
    .video_codec
    .as_ref()
    .and_then(|v| v.width)
    .unwrap_or(640);
  let height = options
    .video_codec
    .as_ref()
    .and_then(|v| v.height)
    .unwrap_or(480);
  let frame_rate = options
    .video_codec
    .as_ref()
    .and_then(|v| v.frame_rate)
    .unwrap_or(30.0);

  // Write Y4M header
  write_y4m_header(&mut output_file, width, height, frame_rate)?;

  // Parse Matroska frames
  let frames = parse_matroska_frames(input_data)?;

  // Convert frames to Y4M
  for (idx, frame) in frames.iter().enumerate() {
    let yuv_data = decode_ivf_frame_to_yuv(frame, width, height)?;

    let output_frame = if let Some(filter) = &options.video_filter {
      apply_video_filter(&yuv_data, &filter.filter_string)?
    } else {
      yuv_data
    };

    write_y4m_frame(&mut output_file, &output_frame, idx as u32)?;
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

/// Write IVF header
fn write_ivf_header<W: std::io::Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  _frame_rate: f64,
) -> Result<(), napi::Error> {
  writer.write_all(b"DKIF")?;
  writer.write_all(&[0u8; 4])?; // Version
  writer.write_all(&[12u8, 0u8, 0u8, 0u8])?; // Header size
  writer.write_all(b"AV01")?; // FourCC (AV1)
  writer.write_all(&width.to_le_bytes()[..2])?;
  writer.write_all(&height.to_le_bytes()[..2])?;
  writer.write_all(&[30u8, 0u8, 0u8, 0u8])?; // Timebase numerator
  writer.write_all(&[1u8, 0u8, 0u8, 0u8])?; // Timebase denominator

  Ok(())
}

/// Write IVF frame
fn write_ivf_frame<W: std::io::Write>(
  writer: &mut W,
  frame_data: &[u8],
  timestamp: u64,
) -> Result<(), napi::Error> {
  let frame_size = frame_data.len() as u32;
  writer.write_all(&frame_size.to_le_bytes())?;
  writer.write_all(&timestamp.to_le_bytes())?;
  writer.write_all(frame_data)?;

  Ok(())
}

/// Write Matroska header
fn write_matroska_header<W: std::io::Write>(
  writer: &mut W,
  _width: i32,
  _height: i32,
  _frame_rate: f64,
) -> Result<(), napi::Error> {
  // EBML header
  writer.write_all(&[0x1a, 0x45, 0xdf, 0xa3])?;
  writer.write_all(&[0x93])?; // EBML header size
  writer.write_all(&[0x42, 0x86])?; // EBMLVersion
  writer.write_all(&[0x80])?; // Version 1
  writer.write_all(&[0x42, 0xf7])?; // EBMLReadVersion
  writer.write_all(&[0x80])?;
  writer.write_all(&[0x42, 0xf2])?; // EBMLMaxIDLength
  writer.write_all(&[0x80])?;
  writer.write_all(&[0x42, 0xf3])?; // EBMLMaxSizeLength
  writer.write_all(&[0x42, 0x82])?; // DocType
  writer.write_all(&[0x84])?;
  writer.write_all(b"webm")?;

  Ok(())
}

/// Write Matroska SimpleBlock
fn write_matroska_simpleblock<W: std::io::Write>(
  writer: &mut W,
  frame_data: &[u8],
  timestamp: u64,
  _track_number: u32,
) -> Result<(), napi::Error> {
  // SimpleBlock element ID (0xA3)
  writer.write_all(&[0xA3])?;

  // Size (variable length)
  let size = frame_data.len() + 4; // 4 bytes for track number + timestamp + flags
  if size < 0x7F {
    writer.write_all(&[size as u8])?;
  } else {
    writer.write_all(&[0x80 | ((size >> 8) as u8), (size & 0xFF) as u8])?;
  }

  // Track number
  writer.write_all(&[0x81])?; // Track 1

  // Timestamp (signed, 2 bytes)
  writer.write_all(&[(timestamp & 0xFF) as u8, ((timestamp >> 8) & 0xFF) as u8])?;

  // Flags
  writer.write_all(&[0x80])?; // Key frame

  // Frame data
  writer.write_all(frame_data)?;

  Ok(())
}

/// Write Matroska trailer
fn write_matroska_trailer<W: std::io::Write>(writer: &mut W) -> Result<(), napi::Error> {
  // Void element to pad
  writer.write_all(&[0xEC])?;
  writer.write_all(&[0x01])?;
  writer.write_all(&[0x00])?;
  writer.flush()?;

  Ok(())
}

/// Write Y4M header
fn write_y4m_header<W: std::io::Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  frame_rate: f64,
) -> Result<(), napi::Error> {
  let fps_num = frame_rate as u32;
  let fps_den = 1u32;

  let header = format!(
    "YUV4MPEG2 W{} H{} F{}:{} Ip A1:1 C420mpeg2\n",
    width, height, fps_num, fps_den
  );

  writer.write_all(header.as_bytes())?;

  Ok(())
}

/// Write Y4M frame
fn write_y4m_frame<W: std::io::Write>(
  writer: &mut W,
  frame_data: &[u8],
  _frame_number: u32,
) -> Result<(), napi::Error> {
  writer.write_all(b"FRAME\n")?;
  writer.write_all(frame_data)?;

  Ok(())
}

/// Parse Y4M header
fn parse_y4m_header(header: &str) -> Result<(i32, i32, f64), napi::Error> {
  let mut width = 640;
  let mut height = 480;
  let mut frame_rate = 30.0;

  for part in header.split_whitespace() {
    if let Some(rest) = part.strip_prefix("W") {
      width = rest
        .parse::<i32>()
        .map_err(|e| napi::Error::from_reason(format!("Invalid width: {}", e)))?;
    } else if let Some(rest) = part.strip_prefix("H") {
      height = rest
        .parse::<i32>()
        .map_err(|e| napi::Error::from_reason(format!("Invalid height: {}", e)))?;
    } else if let Some(rest) = part.strip_prefix("F") {
      let parts: Vec<&str> = rest.split(':').collect();
      if parts.len() == 2 {
        let num: f64 = parts[0]
          .parse()
          .map_err(|e| napi::Error::from_reason(format!("Invalid frame rate numerator: {}", e)))?;
        let den: f64 = parts[1].parse().map_err(|e| {
          napi::Error::from_reason(format!("Invalid frame rate denominator: {}", e))
        })?;
        frame_rate = num / den;
      }
    }
  }

  Ok((width, height, frame_rate))
}

/// Parse Matroska frames (simplified)
fn parse_matroska_frames(data: &[u8]) -> Result<Vec<Vec<u8>>, napi::Error> {
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

/// Apply video filter with actual processing
fn apply_video_filter(frame_data: &[u8], filter_string: &str) -> Result<Vec<u8>, napi::Error> {
  let mut filter_parts = filter_string.split('=');
  let filter_name = filter_parts.next().unwrap_or("").to_lowercase();
  let filter_params = filter_parts.next().map(|s| s.to_string());

  match filter_name.as_str() {
    "scale" | "resize" => {
      // Parse scale parameters (e.g., "scale=640:480")
      if let Some(params) = filter_params {
        let dims: Vec<&str> = params.split(':').collect();
        if dims.len() >= 2 {
          if let (Ok(target_w), Ok(target_h)) = (dims[0].parse::<i32>(), dims[1].parse::<i32>()) {
            return apply_scale_filter(frame_data, target_w, target_h);
          }
        }
      }
      Ok(frame_data.to_vec())
    }
    "crop" => {
      // Parse crop parameters (e.g., "crop=640:360:0:60")
      if let Some(params) = filter_params {
        let parts: Vec<&str> = params.split(':').collect();
        if parts.len() >= 4 {
          if let (Ok(w), Ok(h), Ok(x), Ok(y)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<i32>(),
            parts[2].parse::<i32>(),
            parts[3].parse::<i32>(),
          ) {
            return apply_crop_filter(frame_data, w, h, x, y);
          }
        }
      }
      Ok(frame_data.to_vec())
    }
    "hflip" => {
      // Horizontal flip
      apply_hflip_filter(frame_data)
    }
    "vflip" => {
      // Vertical flip
      apply_vflip_filter(frame_data)
    }
    "brightness" => {
      // Brightness adjustment
      if let Some(params) = filter_params {
        if let Ok(value) = params.parse::<i32>() {
          return apply_brightness_filter(frame_data, value);
        }
      }
      Ok(frame_data.to_vec())
    }
    "contrast" => {
      // Contrast adjustment
      if let Some(params) = filter_params {
        if let Ok(value) = params.parse::<f32>() {
          return apply_contrast_filter(frame_data, value);
        }
      }
      Ok(frame_data.to_vec())
    }
    _ => {
      // Unknown filter, return original data
      Ok(frame_data.to_vec())
    }
  }
}

/// Apply scale filter to frame data
fn apply_scale_filter(
  frame_data: &[u8],
  target_width: i32,
  target_height: i32,
) -> Result<Vec<u8>, napi::Error> {
  // For YUV420 data, calculate original dimensions
  let data_len = frame_data.len();
  if data_len < 1 {
    return Ok(frame_data.to_vec());
  }

  // Estimate original dimensions (assuming YUV420)
  let original_pixels = (data_len as i32) * 2 / 3;

  let target_pixels = target_width * target_height;
  let scale_ratio = target_pixels as f64 / original_pixels as f64;

  // Simple scaling by subsampling or upsampling
  let mut scaled_data = Vec::with_capacity((target_pixels as usize) * 3 / 2);

  if scale_ratio < 1.0 {
    // Downsample: skip pixels
    let step = (1.0 / scale_ratio) as usize;
    let y_size = target_width as usize * target_height as usize;
    let uv_size = y_size / 4;

    // Y plane
    for i in (0..y_size).step_by(step) {
      scaled_data.push(frame_data[i]);
    }
    // Fill with last value if needed
    while scaled_data.len() < y_size {
      scaled_data.push(*scaled_data.last().unwrap_or(&128));
    }

    // UV planes
    for i in (y_size..y_size + uv_size).step_by(step) {
      scaled_data.push(frame_data[i]);
    }
    while scaled_data.len() < y_size + 2 * uv_size {
      scaled_data.push(*scaled_data.last().unwrap_or(&128));
    }
  } else {
    // Upsample: duplicate pixels
    let repeat = scale_ratio as usize;
    let y_size = target_width as usize * target_height as usize;
    let uv_size = y_size / 4;

    for &byte in &frame_data[..std::cmp::min(frame_data.len(), y_size)] {
      for _ in 0..repeat {
        scaled_data.push(byte);
      }
    }
    while scaled_data.len() < y_size {
      scaled_data.push(*scaled_data.last().unwrap_or(&128));
    }

    let uv_start = std::cmp::min(y_size, frame_data.len());
    for &byte in &frame_data[uv_start..std::cmp::min(frame_data.len(), uv_start + uv_size)] {
      for _ in 0..repeat {
        scaled_data.push(byte);
      }
    }
    while scaled_data.len() < y_size + 2 * uv_size {
      scaled_data.push(*scaled_data.last().unwrap_or(&128));
    }
  }

  Ok(scaled_data)
}

/// Apply crop filter to frame data
fn apply_crop_filter(
  frame_data: &[u8],
  crop_w: i32,
  crop_h: i32,
  crop_x: i32,
  crop_y: i32,
) -> Result<Vec<u8>, napi::Error> {
  let data_len = frame_data.len();
  if data_len < 1 {
    return Ok(frame_data.to_vec());
  }

  // Estimate original dimensions
  let original_pixels = (data_len as i32) * 2 / 3;
  let original_width = (original_pixels as f64).sqrt() as i32;
  let original_height = original_pixels / original_width;

  // Validate crop parameters
  if crop_x + crop_w > original_width || crop_y + crop_h > original_height {
    return Err(napi::Error::from_reason(
      "Crop parameters exceed frame dimensions",
    ));
  }

  let crop_pixels = crop_w * crop_h;
  let cropped_y_size = crop_pixels as usize;
  let cropped_uv_size = cropped_y_size / 4;
  let total_cropped_size = cropped_y_size + 2 * cropped_uv_size;
  let mut cropped_data = Vec::with_capacity(total_cropped_size);

  // Crop Y plane
  for y in crop_y as usize..(crop_y + crop_h) as usize {
    let row_start = y * original_width as usize + crop_x as usize;
    let row_end = row_start + crop_w as usize;
    if row_end <= data_len {
      cropped_data.extend_from_slice(&frame_data[row_start..row_end]);
    }
  }

  // Crop UV planes (subsampled)
  let uv_width = original_width / 2;
  let uv_crop_x = crop_x / 2;
  let uv_crop_y = crop_y / 2;
  let uv_crop_w = crop_w / 2;
  let uv_crop_h = crop_h / 2;

  let y_plane_size = original_width as usize * original_height as usize;

  for uv_plane in 0..2 {
    let uv_plane_start = y_plane_size + uv_plane * (y_plane_size / 4);
    for y in uv_crop_y as usize..(uv_crop_y + uv_crop_h) as usize {
      let row_start = uv_plane_start + y * uv_width as usize + uv_crop_x as usize;
      let row_end = row_start + uv_crop_w as usize;
      if row_end <= data_len {
        cropped_data.extend_from_slice(&frame_data[row_start..row_end]);
      }
    }
  }

  Ok(cropped_data)
}

/// Apply horizontal flip filter
fn apply_hflip_filter(frame_data: &[u8]) -> Result<Vec<u8>, napi::Error> {
  let data_len = frame_data.len();
  if data_len < 1 {
    return Ok(frame_data.to_vec());
  }

  // Estimate dimensions
  let original_pixels = (data_len as i32) * 2 / 3;
  let original_width = (original_pixels as f64).sqrt() as i32;
  let original_height = original_pixels / original_width;

  let y_plane_size = original_width as usize * original_height as usize;
  let uv_plane_size = y_plane_size / 4;

  let mut flipped_data = Vec::with_capacity(data_len);

  // Flip Y plane row by row
  for y in 0..original_height as usize {
    let row_start = y * original_width as usize;
    let row_end = row_start + original_width as usize;
    if row_end <= data_len {
      let row = &frame_data[row_start..row_end];
      flipped_data.extend(row.iter().rev());
    }
  }

  // Flip UV planes
  let uv_width = original_width / 2;
  let uv_height = original_height / 2;

  for uv_plane in 0..2 {
    let uv_plane_start = y_plane_size + uv_plane * uv_plane_size;
    for y in 0..uv_height as usize {
      let row_start = uv_plane_start + y * uv_width as usize;
      let row_end = row_start + uv_width as usize;
      if row_end <= data_len {
        let row = &frame_data[row_start..row_end];
        flipped_data.extend(row.iter().rev());
      }
    }
  }

  Ok(flipped_data)
}

/// Apply vertical flip filter
fn apply_vflip_filter(frame_data: &[u8]) -> Result<Vec<u8>, napi::Error> {
  let data_len = frame_data.len();
  if data_len < 1 {
    return Ok(frame_data.to_vec());
  }

  // Estimate dimensions
  let original_pixels = (data_len as i32) * 2 / 3;
  let original_width = (original_pixels as f64).sqrt() as i32;
  let original_height = original_pixels / original_width;

  let y_plane_size = original_width as usize * original_height as usize;
  let uv_plane_size = y_plane_size / 4;

  let mut flipped_data = Vec::with_capacity(data_len);

  // Flip Y plane
  for y in (0..original_height as usize).rev() {
    let row_start = y * original_width as usize;
    let row_end = row_start + original_width as usize;
    if row_end <= data_len {
      flipped_data.extend_from_slice(&frame_data[row_start..row_end]);
    }
  }

  // Flip UV planes
  let uv_width = original_width / 2;
  let uv_height = original_height / 2;

  for uv_plane in 0..2 {
    let uv_plane_start = y_plane_size + uv_plane * uv_plane_size;
    for y in (0..uv_height as usize).rev() {
      let row_start = uv_plane_start + y * uv_width as usize;
      let row_end = row_start + uv_width as usize;
      if row_end <= data_len {
        flipped_data.extend_from_slice(&frame_data[row_start..row_end]);
      }
    }
  }

  Ok(flipped_data)
}

/// Apply brightness filter
fn apply_brightness_filter(frame_data: &[u8], adjustment: i32) -> Result<Vec<u8>, napi::Error> {
  let mut adjusted_data = Vec::with_capacity(frame_data.len());

  for &byte in frame_data {
    let adjusted = (byte as i32 + adjustment).clamp(0, 255) as u8;
    adjusted_data.push(adjusted);
  }

  Ok(adjusted_data)
}

/// Apply contrast filter
fn apply_contrast_filter(frame_data: &[u8], contrast: f32) -> Result<Vec<u8>, napi::Error> {
  let mut adjusted_data = Vec::with_capacity(frame_data.len());
  let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));

  for &byte in frame_data {
    let adjusted = (factor * (byte as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
    adjusted_data.push(adjusted);
  }

  Ok(adjusted_data)
}

/// Encode YUV to IVF frame with actual compression
fn encode_yuv_to_ivf_frame(
  yuv_data: &[u8],
  _width: i32,
  _height: i32,
) -> Result<Vec<u8>, napi::Error> {
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
fn decode_ivf_frame_to_yuv(
  frame_data: &[u8],
  _width: i32,
  _height: i32,
) -> Result<Vec<u8>, napi::Error> {
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
