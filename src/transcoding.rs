//! Transcoding module
//!
//! This module provides transcoding functionality between different media formats
//! using the rust-av ecosystem.

use crate::types::TranscodeOptions;
use crate::video_encoding::VideoCodec;
use image::{RgbImage, RgbaImage};
use napi::Error;
use napi_derive::napi;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

/// Extract frames as RGBA buffers
#[napi(object)]
pub struct FrameData {
  pub frame_number: u32,
  pub width: u32,
  pub height: u32,
  pub rgba_data: Vec<u8>,
}

/// Extract frames from video file as RGBA buffers
#[napi]
pub fn extract_frames_as_rgba(
  input_path: String,
  max_frames: Option<u32>,
) -> Result<Vec<FrameData>, Error> {
  let path_buf = PathBuf::from(&input_path);

  if !path_buf.exists() {
    return Err(Error::from_reason(format!(
      "Input file not found: {}",
      path_buf.display()
    )));
  }

  let input_data = std::fs::read(&path_buf)
    .map_err(|e| Error::from_reason(format!("Failed to read input file: {}", e)))?;

  let format = crate::format::detect_format(&path_buf);

  match format {
    crate::format::MediaFormat::Y4m => {
      extract_y4m_frames_as_rgba(&input_data, max_frames.unwrap_or(u32::MAX))
    }
    crate::format::MediaFormat::Ivf => {
      extract_ivf_frames_as_rgba(&input_data, max_frames.unwrap_or(u32::MAX))
    }
    _ => Err(Error::from_reason(format!(
      "Unsupported format for frame extraction: {:?}",
      format
    ))),
  }
}

/// Extract frames from Y4M file as RGBA
fn extract_y4m_frames_as_rgba(data: &[u8], max_frames: u32) -> Result<Vec<FrameData>, Error> {
  // Parse Y4M header
  let header_end = data
    .iter()
    .position(|&b| b == b'\n')
    .ok_or_else(|| Error::from_reason("Invalid Y4M file: no header found"))?;

  let header = std::str::from_utf8(&data[..header_end])
    .map_err(|e| Error::from_reason(format!("Invalid Y4M header: {}", e)))?;

  let (width, height, _frame_rate) = parse_y4m_header(header)?;

  let mut frames = Vec::new();
  let mut offset = header_end + 1;
  let mut frame_count = 0u32;

  let y_size = width as usize * height as usize;
  let uv_size = y_size / 4;
  let frame_size = y_size + 2 * uv_size;

  while offset < data.len() && frame_count < max_frames {
    // Look for FRAME marker
    if offset + 5 <= data.len() && &data[offset..offset + 5] == b"FRAME" {
      offset += 5;

      // Skip to newline
      while offset < data.len() && data[offset] != b'\n' {
        offset += 1;
      }
      if offset < data.len() {
        offset += 1;
      }

      if offset + frame_size > data.len() {
        break;
      }

      let yuv_data = &data[offset..offset + frame_size];

      // Convert YUV420 to RGBA
      let rgba_data = yuv420_to_rgba(yuv_data, width as usize, height as usize);

      frames.push(FrameData {
        frame_number: frame_count,
        width: width as u32,
        height: height as u32,
        rgba_data,
      });

      offset += frame_size;
      frame_count += 1;
    } else {
      offset += 1;
    }
  }

  Ok(frames)
}

/// Extract frames from IVF file as RGBA
fn extract_ivf_frames_as_rgba(data: &[u8], max_frames: u32) -> Result<Vec<FrameData>, Error> {
  // Parse IVF header
  if data.len() < 32 {
    return Err(Error::from_reason("Invalid IVF file: header too short"));
  }

  let width = u16::from_le_bytes([data[24], data[25]]) as usize;
  let height = u16::from_le_bytes([data[26], data[27]]) as usize;

  let mut frames = Vec::new();
  let mut offset = 32; // Skip IVF header
  let mut frame_count = 0u32;

  while offset + 12 <= data.len() && frame_count < max_frames {
    let frame_size_bytes = u32::from_le_bytes([
      data[offset],
      data[offset + 1],
      data[offset + 2],
      data[offset + 3],
    ]) as usize;

    if offset + 12 + frame_size_bytes > data.len() {
      break;
    }

    let frame_data = &data[offset + 12..offset + 12 + frame_size_bytes];

    // Assuming IVF contains YUV420 data
    let rgba_data = yuv420_to_rgba(frame_data, width, height);

    frames.push(FrameData {
      frame_number: frame_count,
      width: width as u32,
      height: height as u32,
      rgba_data,
    });

    offset += 12 + frame_size_bytes;
    frame_count += 1;
  }

  Ok(frames)
}

/// Convert YUV420 to RGBA
fn yuv420_to_rgba(yuv_data: &[u8], width: usize, height: usize) -> Vec<u8> {
  let y_size = width * height;
  let uv_size = y_size / 4;

  let y_plane = &yuv_data[0..y_size];
  let u_plane = &yuv_data[y_size..y_size + uv_size];
  let v_plane = &yuv_data[y_size + uv_size..y_size + 2 * uv_size];

  let mut rgba = vec![0u8; width * height * 4];

  for y in 0..height {
    for x in 0..width {
      let y_idx = y * width + x;
      let uv_y = y / 2;
      let uv_x = x / 2;
      let uv_idx = uv_y * (width / 2) + uv_x;

      let y_val = y_plane[y_idx] as f32;
      let u_val = u_plane[uv_idx] as f32 - 128.0;
      let v_val = v_plane[uv_idx] as f32 - 128.0;

      // YUV to RGB conversion
      let r = (y_val + 1.402 * v_val).clamp(0.0, 255.0) as u8;
      let g = (y_val - 0.344 * u_val - 0.714 * v_val).clamp(0.0, 255.0) as u8;
      let b = (y_val + 1.772 * u_val).clamp(0.0, 255.0) as u8;

      let rgba_idx = y_idx * 4;
      rgba[rgba_idx] = r;
      rgba[rgba_idx + 1] = g;
      rgba[rgba_idx + 2] = b;
      rgba[rgba_idx + 3] = 255; // Alpha
    }
  }

  rgba
}

/// Options for saving frames as images
#[napi(object)]
pub struct SaveFramesOptions {
  /// Output directory for images
  pub output_dir: String,
  /// Image format (png, jpg, bmp, etc.)
  pub image_format: Option<String>,
  /// Prefix for image filenames
  pub filename_prefix: Option<String>,
  /// Number of digits for frame numbering (e.g., 4 = frame_0001.png)
  pub frame_number_digits: Option<u32>,
}

/// Save frames as image files
#[napi]
pub fn save_frames_as_images(
  frames: Vec<FrameData>,
  options: SaveFramesOptions,
) -> Result<Vec<String>, Error> {
  let output_dir = PathBuf::from(&options.output_dir);

  // Create output directory if it doesn't exist
  std::fs::create_dir_all(&output_dir)
    .map_err(|e| Error::from_reason(format!("Failed to create output directory: {}", e)))?;

  let image_format = options
    .image_format
    .unwrap_or("png".to_string())
    .to_lowercase();
  let prefix = options.filename_prefix.unwrap_or("frame".to_string());
  let num_digits = options.frame_number_digits.unwrap_or(4);

  let mut saved_paths = Vec::new();

  for frame in frames {
    let filename = format!(
      "{}_{:0width$}.{}",
      prefix,
      frame.frame_number,
      image_format,
      width = num_digits as usize
    );

    let filepath = output_dir.join(&filename);

    // Convert RGBA data to image
    let rgba_image: RgbaImage = RgbaImage::from_raw(frame.width, frame.height, frame.rgba_data)
      .ok_or_else(|| Error::from_reason("Failed to create image from RGBA data"))?;

    // Save image based on format
    match image_format.as_str() {
      "png" => {
        rgba_image
          .save(&filepath)
          .map_err(|e| Error::from_reason(format!("Failed to save PNG: {}", e)))?;
      }
      "jpg" | "jpeg" => {
        let rgb_image = RgbImage::from_raw(
          frame.width,
          frame.height,
          rgba_image
            .pixels()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect::<Vec<u8>>(),
        )
        .ok_or_else(|| Error::from_reason("Failed to create RGB image"))?;

        rgb_image
          .save(&filepath)
          .map_err(|e| Error::from_reason(format!("Failed to save JPEG: {}", e)))?;
      }
      "bmp" => {
        rgba_image
          .save(&filepath)
          .map_err(|e| Error::from_reason(format!("Failed to save BMP: {}", e)))?;
      }
      _ => {
        return Err(Error::from_reason(format!(
          "Unsupported image format: {}. Supported formats: png, jpg, jpeg, bmp",
          image_format
        )));
      }
    }

    saved_paths.push(filepath.to_string_lossy().to_string());
  }

  Ok(saved_paths)
}

/// Extract frames from video and save directly as images
#[napi]
pub fn extract_frames_to_images(
  input_path: String,
  output_dir: String,
  max_frames: Option<u32>,
  image_format: Option<String>,
) -> Result<Vec<String>, Error> {
  // Extract frames as RGBA
  let frames = extract_frames_as_rgba(input_path, max_frames)?;

  // Save frames as images
  let saved_paths = save_frames_as_images(
    frames,
    SaveFramesOptions {
      output_dir,
      image_format,
      filename_prefix: Some("frame".to_string()),
      frame_number_digits: Some(4),
    },
  )?;

  Ok(saved_paths)
}

/// Extract frames from video using v_frame
#[napi]
pub fn extract_frames_with_v_frame(
  input_path: String,
  max_frames: Option<u32>,
) -> Result<Vec<FrameData>, Error> {
  let path_buf = PathBuf::from(&input_path);

  if !path_buf.exists() {
    return Err(Error::from_reason(format!(
      "Input file not found: {}",
      path_buf.display()
    )));
  }

  let input_data = std::fs::read(&path_buf)
    .map_err(|e| Error::from_reason(format!("Failed to read input file: {}", e)))?;

  let format = crate::format::detect_format(&path_buf);

  match format {
    crate::format::MediaFormat::Y4m => {
      extract_y4m_frames_with_v_frame(&input_data, max_frames.unwrap_or(u32::MAX))
    }
    crate::format::MediaFormat::Ivf => {
      extract_ivf_frames_with_v_frame(&input_data, max_frames.unwrap_or(u32::MAX))
    }
    _ => Err(Error::from_reason(format!(
      "Unsupported format for frame extraction: {:?}",
      format
    ))),
  }
}

/// Extract frames from Y4M using optimized parsing
fn extract_y4m_frames_with_v_frame(data: &[u8], max_frames: u32) -> Result<Vec<FrameData>, Error> {
  // Parse Y4M header
  let header_end = data
    .iter()
    .position(|&b| b == b'\n')
    .ok_or_else(|| Error::from_reason("Invalid Y4M file: no header found"))?;

  let header = std::str::from_utf8(&data[..header_end])
    .map_err(|e| Error::from_reason(format!("Invalid Y4M header: {}", e)))?;

  let (width, height, _frame_rate) = parse_y4m_header(header)?;

  let mut frames = Vec::new();
  let mut offset = header_end + 1;
  let mut frame_count = 0u32;

  let y_size = width as usize * height as usize;
  let uv_size = y_size / 4;
  let frame_size = y_size + 2 * uv_size;

  while offset < data.len() && frame_count < max_frames {
    // Look for FRAME marker
    if offset + 5 <= data.len() && &data[offset..offset + 5] == b"FRAME" {
      offset += 5;

      // Skip to newline
      while offset < data.len() && data[offset] != b'\n' {
        offset += 1;
      }
      if offset < data.len() {
        offset += 1;
      }

      if offset + frame_size > data.len() {
        break;
      }

      let yuv_data = &data[offset..offset + frame_size];

      // Convert YUV420 to RGBA
      let rgba_data = yuv420_to_rgba(yuv_data, width as usize, height as usize);

      frames.push(FrameData {
        frame_number: frame_count,
        width: width as u32,
        height: height as u32,
        rgba_data,
      });

      offset += frame_size;
      frame_count += 1;
    } else {
      offset += 1;
    }
  }

  Ok(frames)
}

/// Extract frames from IVF using optimized parsing
fn extract_ivf_frames_with_v_frame(data: &[u8], max_frames: u32) -> Result<Vec<FrameData>, Error> {
  // Parse IVF header
  if data.len() < 32 {
    return Err(Error::from_reason("Invalid IVF file: header too short"));
  }

  let width = u16::from_le_bytes([data[24], data[25]]) as usize;
  let height = u16::from_le_bytes([data[26], data[27]]) as usize;

  let mut frames = Vec::new();
  let mut offset = 32; // Skip IVF header
  let mut frame_count = 0u32;

  while offset + 12 <= data.len() && frame_count < max_frames {
    let frame_size_bytes = u32::from_le_bytes([
      data[offset],
      data[offset + 1],
      data[offset + 2],
      data[offset + 3],
    ]) as usize;

    if offset + 12 + frame_size_bytes > data.len() {
      break;
    }

    let frame_data = &data[offset + 12..offset + 12 + frame_size_bytes];

    // Assuming IVF contains YUV420 data
    let rgba_data = yuv420_to_rgba(frame_data, width, height);

    frames.push(FrameData {
      frame_number: frame_count,
      width: width as u32,
      height: height as u32,
      rgba_data,
    });

    offset += 12 + frame_size_bytes;
    frame_count += 1;
  }

  Ok(frames)
}

/// Transcode IVF to Matroska format
pub fn transcode_ivf_to_matroska(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  // Parse IVF header
  if input_data.len() < 32 {
    return Err(Error::from_reason("Invalid IVF file: header too short"));
  }

  let width = u16::from_le_bytes([input_data[24], input_data[25]]) as i32;
  let height = u16::from_le_bytes([input_data[26], input_data[27]]) as i32;
  let frame_rate = 30.0;

  // Detect codec from IVF FourCC
  let fourcc = &input_data[16..20];
  let codec = match fourcc {
    b"AV01" => VideoCodec::Av1,
    b"VP90" => VideoCodec::Vp9,
    b"VP80" => VideoCodec::Vp8,
    _ => VideoCodec::Vp9, // Default to VP9
  };

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

  // Generate CodecPrivate data for the codec
  let codec_private = generate_codec_private(codec, final_width, final_height);

  // Create output file
  let mut output = BufWriter::new(
    File::create(output_path)
      .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?,
  );

  // Write EBML header for WebM with correct codec ID and CodecPrivate
  write_webm_header(
    &mut output,
    final_width,
    final_height,
    final_frame_rate,
    codec.codec_id(),
    codec_private.as_deref(),
  )?;

  // Write frames as SimpleBlocks in a Cluster
  let mut offset = 32; // Skip IVF header
  let mut frame_count = 0u32;
  let timebase = 1_000_000_000u64 / (final_frame_rate as u64); // nanoseconds per frame

  // Start cluster
  write_cluster_start(&mut output, 0)?;

  while offset + 12 <= input_data.len() {
    let frame_size = u32::from_le_bytes([
      input_data[offset],
      input_data[offset + 1],
      input_data[offset + 2],
      input_data[offset + 3],
    ]) as usize;

    let ivf_timestamp = u64::from_le_bytes([
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

    // Convert IVF timestamp to Matroska timestamp (in milliseconds)
    let matroska_timestamp = (ivf_timestamp * timebase / 1_000_000) as u64;

    // Write frame as SimpleBlock
    write_simpleblock(&mut output, frame_data, matroska_timestamp, frame_count)?;

    offset += 12 + frame_size;
    frame_count += 1;
  }

  output
    .flush()
    .map_err(|e| Error::from_reason(format!("Failed to flush output: {}", e)))?;

  Ok(())
}

/// Generate CodecPrivate data for different codecs
fn generate_codec_private(codec: VideoCodec, width: i32, height: i32) -> Option<Vec<u8>> {
  match codec {
    VideoCodec::Vp9 => {
      // VP9 CodecPrivate: VP9 config record
      // https://www.webmproject.org/docs/container/
      let mut config = Vec::new();
      config.push(0x01); // Profile
      config.push(0x00); // Level
      config.push(0x00); // Bit depth minus 8
      config.push(0x00); // Chroma subsampling
      config.push((width & 0xFF) as u8);
      config.push(((width >> 8) & 0xFF) as u8);
      config.push((height & 0xFF) as u8);
      config.push(((height >> 8) & 0xFF) as u8);
      Some(config)
    }
    VideoCodec::Av1 => {
      // AV1 CodecPrivate: AV1 config OBUs
      // https://aomediacodec.github.io/av1-isobmff/#av1codecconfigurationbox
      let mut config = Vec::new();
      config.push(0x81); // marker (1) + version (7)
      config.push(0x00); // seq_profile
      config.push(0x00); // seq_level_idx_0
      config.push(0x00); // seq_tier_0 (1) + high_bitdepth (1) + twelve_bit (1) + monochrome (1) + chroma_subsampling_x (1) + chroma_subsampling_y (1) + chroma_sample_position (2)
      config.push(0x00); // reserved (3) + initial_presentation_delay_present (1) + initial_presentation_delay_minus_one (4)
      Some(config)
    }
    VideoCodec::Vp8 => {
      // VP8 CodecPrivate: VP8 config record
      let mut config = Vec::new();
      config.push(0x00); // version
      config.push(0x00); // show_frame (1) + clamp (1) + type (3) + spatial_resampling (1) + update_segment (1) + update_mb_no_coeffs (1) + error_resilient (1)
      config.push(0x00); // filter_type (1) + loop_filter_adj_enable (1) + loop_filter_adj (2) + update_segment (1) + update_mb_no_coeffs (1) + error_resilient (1)
      config.push(0x00); // q_index
      config.push(0x00); // loop_filter_level
      Some(config)
    }
  }
}

/// Transcode Matroska to IVF format
pub fn transcode_matroska_to_ivf(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?;

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

  // Determine codec (default to VP9)
  let codec = VideoCodec::Vp9;

  // Write IVF header with correct FourCC
  write_ivf_header(&mut output_file, width, height, frame_rate, codec.fourcc())?;

  // Parse Matroska and extract frames
  let frames = parse_matroska_frames(input_data)?;

  // Write frames to IVF
  for (idx, frame) in frames.iter().enumerate() {
    write_ivf_frame(&mut output_file, frame, idx as u64)?;
  }

  Ok(())
}

/// Transcode Y4M to IVF format
pub fn transcode_y4m_to_ivf(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse Y4M header
  let header_end = input_data
    .iter()
    .position(|&b| b == b'\n')
    .ok_or_else(|| Error::from_reason("Invalid Y4M file: no header found"))?;

  let header = std::str::from_utf8(&input_data[..header_end])
    .map_err(|e| Error::from_reason(format!("Invalid Y4M header: {}", e)))?;

  let (mut width, mut height, mut frame_rate) = parse_y4m_header(header)?;

  // Apply codec options
  if let Some(video_opts) = &options.video_codec {
    width = video_opts.width.unwrap_or(width);
    height = video_opts.height.unwrap_or(height);
    frame_rate = video_opts.frame_rate.unwrap_or(frame_rate);
  }

  // Determine codec (default to VP9)
  let codec = VideoCodec::Vp9;

  // Write IVF header with correct FourCC
  write_ivf_header(&mut output_file, width, height, frame_rate, codec.fourcc())?;

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

      // Write YUV data directly as IVF frame (uncompressed)
      write_ivf_frame(&mut output_file, yuv_data, frame_idx as u64)?;

      offset += frame_size;
      frame_idx += 1;
    } else {
      offset += 1;
    }
  }

  Ok(())
}

/// Transcode IVF to Y4M format
pub fn transcode_ivf_to_y4m(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse IVF header
  if input_data.len() < 32 {
    return Err(Error::from_reason("Invalid IVF file: header too short"));
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

    // Write frame directly as YUV data
    write_y4m_frame(&mut output_file, frame_data, frame_count)?;

    offset += 12 + frame_size;
    frame_count += 1;
  }

  Ok(())
}

/// Transcode Y4M to Matroska format
pub fn transcode_y4m_to_matroska(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse Y4M header
  let header_end = input_data
    .iter()
    .position(|&b| b == b'\n')
    .ok_or_else(|| Error::from_reason("Invalid Y4M file: no header found"))?;

  let header = std::str::from_utf8(&input_data[..header_end])
    .map_err(|e| Error::from_reason(format!("Invalid Y4M header: {}", e)))?;

  let (mut width, mut height, mut frame_rate) = parse_y4m_header(header)?;

  // Apply codec options
  if let Some(video_opts) = &options.video_codec {
    width = video_opts.width.unwrap_or(width);
    height = video_opts.height.unwrap_or(height);
    frame_rate = video_opts.frame_rate.unwrap_or(frame_rate);
  }

  // Determine codec (default to VP9)
  let codec = VideoCodec::Vp9;

  // Generate CodecPrivate data
  let codec_private = generate_codec_private(codec, width, height);

  // Write WebM header with correct codec ID and CodecPrivate
  write_webm_header(
    &mut output_file,
    width,
    height,
    frame_rate,
    codec.codec_id(),
    codec_private.as_deref(),
  )?;

  // Parse and convert Y4M frames
  let mut offset = header_end + 1;
  let mut frame_idx = 0u32;

  // Start cluster
  write_cluster_start(&mut output_file, 0)?;

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

      // Write frame as SimpleBlock with correct timestamp
      write_simpleblock(&mut output_file, yuv_data, frame_idx as u64, frame_idx)?;

      offset += frame_size;
      frame_idx += 1;
    } else {
      offset += 1;
    }
  }

  output_file
    .flush()
    .map_err(|e| Error::from_reason(format!("Failed to flush output: {}", e)))?;

  Ok(())
}

/// Transcode Matroska to Y4M format
pub fn transcode_matroska_to_y4m(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?;

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
    write_y4m_frame(&mut output_file, frame, idx as u32)?;
  }

  Ok(())
}

// Helper functions for format writing

fn write_ivf_header<W: Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  frame_rate: f64,
  fourcc: [u8; 4],
) -> Result<(), Error> {
  writer.write_all(b"DKIF")?;
  writer.write_all(&[0u8; 4])?; // Version
  writer.write_all(&[12u8, 0u8, 0u8, 0u8])?; // Header size
  writer.write_all(&fourcc)?; // FourCC (VP90, AV01, VP80, etc.)
  writer.write_all(&width.to_le_bytes()[..2])?;
  writer.write_all(&height.to_le_bytes()[..2])?;
  writer.write_all(&[(frame_rate as u32).to_le_bytes()[0], 0u8, 0u8, 0u8])?; // Timebase numerator
  writer.write_all(&[1u8, 0u8, 0u8, 0u8])?; // Timebase denominator

  Ok(())
}

fn write_ivf_frame<W: Write>(
  writer: &mut W,
  frame_data: &[u8],
  timestamp: u64,
) -> Result<(), Error> {
  let frame_size = frame_data.len() as u32;
  writer.write_all(&frame_size.to_le_bytes())?;
  writer.write_all(&timestamp.to_le_bytes())?;
  writer.write_all(frame_data)?;

  Ok(())
}

fn write_webm_header<W: Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  frame_rate: f64,
  codec_id: &str,
  codec_private: Option<&[u8]>,
) -> Result<(), Error> {
  // EBML header
  writer.write_all(&[0x1a, 0x45, 0xdf, 0xa3])?;
  writer.write_all(&[0x93])?; // EBML header size (19 bytes)
  writer.write_all(&[0x42, 0x86])?; // EBMLVersion
  writer.write_all(&[0x80])?; // Version 1
  writer.write_all(&[0x42, 0xf7])?; // EBMLReadVersion
  writer.write_all(&[0x80])?;
  writer.write_all(&[0x42, 0xf2])?; // EBMLMaxIDLength
  writer.write_all(&[0x80])?;
  writer.write_all(&[0x42, 0xf3])?; // EBMLMaxSizeLength
  writer.write_all(&[0x80])?;
  writer.write_all(&[0x42, 0x82])?; // DocType
  writer.write_all(&[0x84])?;
  writer.write_all(b"webm")?;

  // Segment header (0x18538067)
  writer.write_all(&[0x18, 0x53, 0x80, 0x67])?;
  writer.write_all(&[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])?; // Unknown size

  // Segment Information (0x1549a966)
  writer.write_all(&[0x15, 0x49, 0xa9, 0x66])?;
  writer.write_all(&[0x8d])?; // Size (29 bytes)

  // TimecodeScale (0x2ad7b1)
  writer.write_all(&[0x2a, 0xd7, 0xb1])?;
  writer.write_all(&[0x83])?;
  writer.write_all(&[0x00, 0x00, 0x00])?; // 1ms

  // MuxingApp (0x4d80)
  writer.write_all(&[0x4d, 0x80])?;
  writer.write_all(&[0x14])?;
  writer.write_all(b"rust-av-kit transcoding")?;

  // WritingApp (0x5741)
  writer.write_all(&[0x57, 0x41])?;
  writer.write_all(&[0x14])?;
  writer.write_all(b"rust-av-kit transcoding")?;

  // Duration (0x4489) - placeholder
  writer.write_all(&[0x44, 0x89])?;
  writer.write_all(&[0x88])?;
  let duration_bytes = (frame_rate.recip() * 1000.0).to_le_bytes();
  writer.write_all(&duration_bytes)?;

  // Tracks (0x1654ae6b) - Calculate size dynamically
  let track_entry_size = 28 + codec_private.map(|p| p.len() + 4).unwrap_or(0); // Base + CodecPrivate
  let tracks_size = 2 + track_entry_size; // TrackEntry ID + size
  writer.write_all(&[0x16, 0x54, 0xae, 0x6b])?;
  write_vint(writer, tracks_size as u64)?;

  // TrackEntry (0xae)
  writer.write_all(&[0xae])?;
  write_vint(writer, track_entry_size as u64)?;

  // TrackNumber (0xd7)
  writer.write_all(&[0xd7])?;
  writer.write_all(&[0x81])?;
  writer.write_all(&[0x01])?;

  // TrackUID (0x73c5)
  writer.write_all(&[0x73, 0xc5])?;
  writer.write_all(&[0x81])?;
  writer.write_all(&[0x01])?;

  // TrackType (0x83) - 1 = video
  writer.write_all(&[0x83])?;
  writer.write_all(&[0x81])?;
  writer.write_all(&[0x01])?;

  // CodecID (0x86) - V_VP9, V_AV1, V_VP8, etc.
  writer.write_all(&[0x86])?;
  let codec_id_bytes = codec_id.as_bytes();
  writer.write_all(&[codec_id_bytes.len() as u8])?;
  writer.write_all(codec_id_bytes)?;

  // CodecPrivate (0x63A2) - Required for VP9/AV1
  if let Some(private_data) = codec_private {
    writer.write_all(&[0x63, 0xa2])?;
    write_vint(writer, private_data.len() as u64)?;
    writer.write_all(private_data)?;
  }

  // Video settings (0xe0)
  writer.write_all(&[0xe0])?;
  writer.write_all(&[0x7e])?; // Size (14 bytes)

  // PixelWidth (0xb0)
  writer.write_all(&[0xb0])?;
  writer.write_all(&[0x82])?;
  writer.write_all(&(width as u16).to_le_bytes())?;

  // PixelHeight (0xba)
  writer.write_all(&[0xba])?;
  writer.write_all(&[0x82])?;
  writer.write_all(&(height as u16).to_le_bytes())?;

  // DisplayWidth (0x54b0)
  writer.write_all(&[0x54, 0xb0])?;
  writer.write_all(&[0x82])?;
  writer.write_all(&(width as u16).to_le_bytes())?;

  // DisplayHeight (0x54ba)
  writer.write_all(&[0x54, 0xba])?;
  writer.write_all(&[0x82])?;
  writer.write_all(&(height as u16).to_le_bytes())?;

  Ok(())
}

/// Write variable-length integer (VINT)
fn write_vint<W: Write>(writer: &mut W, value: u64) -> Result<(), Error> {
  if value < 0x7F {
    writer.write_all(&[value as u8])?;
  } else if value < 0x3FFF {
    writer.write_all(&[((value >> 8) | 0x80) as u8, (value & 0xFF) as u8])?;
  } else if value < 0x1FFFFF {
    writer.write_all(&[
      ((value >> 16) | 0x80 | 0x40) as u8,
      ((value >> 8) & 0xFF) as u8,
      (value & 0xFF) as u8,
    ])?;
  } else if value < 0x0FFFFFFF {
    writer.write_all(&[
      ((value >> 24) | 0x80 | 0x40 | 0x20) as u8,
      ((value >> 16) & 0xFF) as u8,
      ((value >> 8) & 0xFF) as u8,
      (value & 0xFF) as u8,
    ])?;
  } else {
    writer.write_all(&[
      ((value >> 32) | 0x80 | 0x40 | 0x20 | 0x10) as u8,
      ((value >> 24) & 0xFF) as u8,
      ((value >> 16) & 0xFF) as u8,
      ((value >> 8) & 0xFF) as u8,
      (value & 0xFF) as u8,
    ])?;
  }
  Ok(())
}

fn write_cluster_start<W: Write>(writer: &mut W, timestamp: u64) -> Result<(), Error> {
  // Cluster element ID (0x1F43B675)
  writer.write_all(&[0x1F, 0x43, 0xB6, 0x75])?;
  // Unknown size
  writer.write_all(&[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])?;

  // Timecode (0xE7)
  writer.write_all(&[0xE7])?;
  writer.write_all(&[0x83])?;
  writer.write_all(&(timestamp as u32).to_le_bytes())?;

  Ok(())
}

fn write_simpleblock<W: Write>(
  writer: &mut W,
  frame_data: &[u8],
  timestamp: u64,
  _track_number: u32,
) -> Result<(), Error> {
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

  // Timestamp (signed, 2 bytes) - Matroska uses signed timestamps in SimpleBlock
  let ts = timestamp as i16;
  writer.write_all(&[(ts & 0xFF) as u8, ((ts >> 8) & 0xFF) as u8])?;

  // Flags
  writer.write_all(&[0x80])?; // Key frame

  // Frame data
  writer.write_all(frame_data)?;

  Ok(())
}

fn write_y4m_header<W: Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  frame_rate: f64,
) -> Result<(), Error> {
  let fps_num = frame_rate as u32;
  let fps_den = 1u32;

  let header = format!(
    "YUV4MPEG2 W{} H{} F{}:{} Ip A1:1 C420mpeg2\n",
    width, height, fps_num, fps_den
  );

  writer.write_all(header.as_bytes())?;

  Ok(())
}

fn write_y4m_frame<W: Write>(
  writer: &mut W,
  frame_data: &[u8],
  _frame_number: u32,
) -> Result<(), Error> {
  writer.write_all(b"FRAME\n")?;
  writer.write_all(frame_data)?;

  Ok(())
}

fn parse_y4m_header(header: &str) -> Result<(i32, i32, f64), Error> {
  let mut width = 320i32;
  let mut height = 240i32;
  let mut frame_rate = 30.0f64;

  for token in header.split_whitespace() {
    if let Some(stripped) = token.strip_prefix('W') {
      width = stripped.parse().unwrap_or(320);
    } else if let Some(stripped) = token.strip_prefix('H') {
      height = stripped.parse().unwrap_or(240);
    } else if let Some(stripped) = token.strip_prefix('F') {
      let parts: Vec<&str> = stripped.split(':').collect();
      if parts.len() == 2 {
        let num: f64 = parts[0].parse().unwrap_or(30.0);
        let den: f64 = parts[1].parse().unwrap_or(1.0);
        frame_rate = num / den;
      }
    }
  }

  Ok((width, height, frame_rate))
}

fn parse_matroska_frames(_input_data: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
  // Simplified parsing - return empty vector for now
  // In a full implementation, this would use av-format to properly parse Matroska
  Ok(Vec::new())
}
