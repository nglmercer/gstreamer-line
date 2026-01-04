//! Transcoding module
//!
//! This module provides transcoding functionality between different media formats.

use crate::encoding::{decode_ivf_frame_to_yuv, encode_yuv_to_ivf_frame};
use crate::format_parsers::{parse_matroska_frames, parse_y4m_header};
use crate::format_writers::{
  write_ivf_frame, write_ivf_header, write_matroska_header,
  write_matroska_simpleblock, write_matroska_trailer, write_y4m_frame,
  write_y4m_header,
};
use crate::types::TranscodeOptions;
use crate::video_filters::apply_video_filter;
use napi::Error;
use std::path::PathBuf;

/// Transcode IVF to Matroska format
pub fn transcode_ivf_to_matroska(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = std::fs::File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?;

  // Parse IVF header
  if input_data.len() < 32 {
    return Err(Error::from_reason("Invalid IVF file: header too short"));
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
pub fn transcode_matroska_to_ivf(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = std::fs::File::create(output_path)
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
pub fn transcode_y4m_to_ivf(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = std::fs::File::create(output_path)
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
pub fn transcode_ivf_to_y4m(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = std::fs::File::create(output_path)
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
pub fn transcode_y4m_to_matroska(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = std::fs::File::create(output_path)
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
pub fn transcode_matroska_to_y4m(
  input_data: &[u8],
  output_path: &PathBuf,
  options: &TranscodeOptions,
) -> Result<(), Error> {
  let mut output_file = std::fs::File::create(output_path)
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
