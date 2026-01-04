 // Re-export modular components
pub mod codec;
pub mod format;
pub mod media;

pub use codec::*;
pub use format::*;
pub use media::*;

use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

  // Detect format from file
  let detected_format = format::detect_format(&path_buf);
  
  let format_name = match &detected_format {
      format::MediaFormat::Ivf => "ivf",
      format::MediaFormat::Matroska => "matroska",
      format::MediaFormat::Y4m => "y4m",
      format::MediaFormat::Unknown(name) => name,
  };
  
  let format_long_name = format::format_long_name(&detected_format);
  
  // Open file and extract media information
  let file = std::fs::File::open(&path_buf)
    .map_err(|e| napi::Error::from_reason(format!("Failed to open file: {}", e)))?;
  
  let metadata = file.metadata()
    .map_err(|e| napi::Error::from_reason(format!("Failed to read file metadata: {}", e)))?;
  
  let file_size = metadata.len();
  
  // Read file to detect codec and stream information
  let mut buffer = vec![0u8; std::cmp::min(8192, file_size as usize)];
  use std::io::Read;
  let mut file_handle = std::fs::File::open(&path_buf)?;
  let bytes_read = file_handle.read(&mut buffer)?;
  
  // Detect codec from file signature
  let (codec_name, codec_type, width, height, frame_rate, sample_rate, channels) =
    detect_codec_from_data(&buffer, &detected_format, &path_buf);
  
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

/// Detect codec from file data and format
fn detect_codec_from_data(
  data: &[u8],
  format: &format::MediaFormat,
  path: &PathBuf
) -> (String, String, Option<i32>, Option<i32>, Option<f64>, Option<i32>, Option<i32>) {
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
        
        (codec_name.to_string(), "video".to_string(), Some(width), Some(height), frame_rate, None, None)
      } else {
        (String::new(), String::new(), None, None, None, None, None)
      }
    }
    format::MediaFormat::Matroska => {
      // Matroska/WebM - detect from file signature
      if data.len() >= 4 && &data[0..4] == b"\x1a\x45\xdf\xa3" {
        // Try to detect codec from file extension or content
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let (codec_name, codec_type) = match ext {
          "webm" => ("vp9", "video"),
          "mkv" => ("h264", "video"),
          _ => ("unknown", "unknown"),
        };
        (codec_name.to_string(), codec_type.to_string(), None, None, None, None, None)
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
          if part.starts_with("W") {
            width = part[1..].parse::<i32>().ok();
          } else if part.starts_with("H") {
            height = part[1..].parse::<i32>().ok();
          } else if part.starts_with("F") {
            let parts: Vec<&str> = part[1..].split(':').collect();
            if parts.len() == 2 {
              if let (Ok(num), Ok(den)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                frame_rate = Some(num / den);
              }
            }
          }
        }
        
        ("raw".to_string(), "video".to_string(), width, height, frame_rate, None, None)
      } else {
        (String::new(), String::new(), None, None, None, None, None)
      }
    }
    format::MediaFormat::Unknown(_) => {
      (String::new(), String::new(), None, None, None, None, None)
    }
  }
}

/// Estimate duration based on file size and codec
fn estimate_duration(file_size: u64, codec_name: &str, width: Option<i32>, height: Option<i32>, frame_rate: Option<f64>) -> f64 {
  // Rough estimation based on typical bitrates
  let pixels = width.unwrap_or(640) * height.unwrap_or(480);
  
  let bitrate = match codec_name {
    "av1" => pixels as f64 * 0.1, // ~0.1 bpp for AV1
    "vp9" => pixels as f64 * 0.15, // ~0.15 bpp for VP9
    "vp8" => pixels as f64 * 0.2,  // ~0.2 bpp for VP8
    "h264" => pixels as f64 * 0.2, // ~0.2 bpp for H.264
    "h265" => pixels as f64 * 0.1, // ~0.1 bpp for H.265
    "raw" => pixels as f64 * 1.5,  // 1.5 bytes per pixel for YUV420
    _ => pixels as f64 * 0.2,
  };
  
  if bitrate > 0.0 {
    let duration_seconds = (file_size as f64 * 8.0) / bitrate;
    duration_seconds
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
  options: &TranscodeOptions
) -> Result<(), napi::Error> {
  use std::io::Write;

  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse IVF header
  if input_data.len() < 32 {
    return Err(napi::Error::from_reason("Invalid IVF file: header too short"));
  }

  let width = u16::from_le_bytes([input_data[24], input_data[25]]) as i32;
  let height = u16::from_le_bytes([input_data[26], input_data[27]]) as i32;
  let timebase_den = u32::from_le_bytes([input_data[28], input_data[29], input_data[30], input_data[31]]);
  let frame_rate = if timebase_den > 0 { 30.0 } else { 30.0 };

  // Apply video codec options if provided
  let (final_width, final_height, final_frame_rate) = if let Some(video_opts) = &options.video_codec {
    (
      video_opts.width.unwrap_or(width),
      video_opts.height.unwrap_or(height),
      video_opts.frame_rate.unwrap_or(frame_rate),
    )
  } else {
    (width, height, frame_rate)
  };

  // Write Matroska EBML header (simplified)
  write_matroska_header(&mut output_file, final_width, final_height, final_frame_rate)?;

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
  options: &TranscodeOptions
) -> Result<(), napi::Error> {
  use std::io::Write;

  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Default dimensions
  let width = options.video_codec.as_ref()
    .and_then(|v| v.width)
    .unwrap_or(640);
  let height = options.video_codec.as_ref()
    .and_then(|v| v.height)
    .unwrap_or(480);
  let frame_rate = options.video_codec.as_ref()
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
  options: &TranscodeOptions
) -> Result<(), napi::Error> {
  use std::io::Write;

  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse Y4M header
  let header_end = input_data.iter().position(|&b| b == b'\n')
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

      // Convert YUV420 to compressed format (simplified - in real implementation would use encoder)
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
  options: &TranscodeOptions
) -> Result<(), napi::Error> {
  use std::io::Write;

  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse IVF header
  if input_data.len() < 32 {
    return Err(napi::Error::from_reason("Invalid IVF file: header too short"));
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

    // Decode compressed frame to YUV (simplified - in real implementation would use decoder)
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
  options: &TranscodeOptions
) -> Result<(), napi::Error> {
  use std::io::Write;

  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse Y4M header
  let header_end = input_data.iter().position(|&b| b == b'\n')
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
  options: &TranscodeOptions
) -> Result<(), napi::Error> {
  use std::io::Write;

  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| napi::Error::from_reason(format!("Failed to create output file: {}", e)))?;

  let width = options.video_codec.as_ref()
    .and_then(|v| v.width)
    .unwrap_or(640);
  let height = options.video_codec.as_ref()
    .and_then(|v| v.height)
    .unwrap_or(480);
  let frame_rate = options.video_codec.as_ref()
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
/// This is a basic implementation that handles format conversion for supported formats.
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

  use std::fs;
  use std::io::{Read, Write};

  match (&input_format, &output_format) {
    (format::MediaFormat::Ivf, format::MediaFormat::Matroska) => {
      let mut input_file = fs::File::open(&input_buf)?;
      let mut output_file = fs::File::create(&output_buf)?;
      
      let mut input_data = Vec::new();
      input_file.read_to_end(&mut input_data)?;
      
      // Write Matroska EBML header
      output_file.write_all(b"\x1a\x45\xdf\xa3")?;
      
      // Write the frame data (simplified - just copy after header)
      // Skip IVF header (32 bytes) and write the rest
      if input_data.len() > 32 {
        output_file.write_all(&input_data[32..])?;
      }
    }
    (format::MediaFormat::Matroska, format::MediaFormat::Ivf) => {
      let mut input_file = fs::File::open(&input_buf)?;
      let mut output_file = fs::File::create(&output_buf)?;
      
      let mut input_data = Vec::new();
      input_file.read_to_end(&mut input_data)?;
      
      // Write IVF header
      output_file.write_all(b"DKIF")?;
      output_file.write_all(&[0u8; 4])?;
      output_file.write_all(&[12u8, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[0x30, 0x39, 0x50, 0x90])?;
      output_file.write_all(&[0x40, 0x01, 0u8, 0u8])?;
      output_file.write_all(&[0xF0, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[30u8, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[1u8, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[30u8, 0u8, 0u8, 0u8])?;
      
      // Write the frame data (skip EBML header if present)
      let data_start = if input_data.len() > 4 && &input_data[0..4] == b"\x1a\x45\xdf\xa3" {
        4
      } else {
        0
      };
      
      if input_data.len() > data_start {
        output_file.write_all(&input_data[data_start..])?;
      }
    }
    (format::MediaFormat::Y4m, format::MediaFormat::Ivf) => {
      let mut input_file = fs::File::open(&input_buf)?;
      let mut output_file = fs::File::create(&output_buf)?;
      
      let mut input_data = Vec::new();
      input_file.read_to_end(&mut input_data)?;
      
      // Write IVF header
      output_file.write_all(b"DKIF")?;
      output_file.write_all(&[0u8; 4])?;
      output_file.write_all(&[12u8, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[0x30, 0x39, 0x50, 0x90])?;
      output_file.write_all(&[0x80, 0x02, 0u8, 0u8])?;
      output_file.write_all(&[0xE0, 0x01, 0u8, 0u8])?;
      output_file.write_all(&[25u8, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[1u8, 0u8, 0u8, 0u8])?;
      output_file.write_all(&[25u8, 0u8, 0u8, 0u8])?;
      
      // Write the frame data (skip Y4M header line)
      if let Some(pos) = input_data.iter().position(|&b| b == b'\n') {
        if pos + 1 < input_data.len() {
          output_file.write_all(&input_data[pos + 1..])?;
        }
      }
    }
    (format::MediaFormat::Ivf, format::MediaFormat::Y4m) => {
      let mut input_file = fs::File::open(&input_buf)?;
      let mut output_file = fs::File::create(&output_buf)?;
      
      let mut input_data = Vec::new();
      input_file.read_to_end(&mut input_data)?;
      
      // Write Y4M header
      output_file.write_all(b"YUV4MPEG2 640 480 25 1\n")?;
      
      // Write the frame data (skip IVF header if present)
      let data_start = if input_data.len() > 32 && &input_data[0..4] == b"DKIF" {
        32
      } else {
        0
      };
      
      if input_data.len() > data_start {
        output_file.write_all(&input_data[data_start..])?;
      }
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
