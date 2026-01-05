//! Transcoding module
//!
//! This module provides transcoding functionality between different media formats
//! using the rust-av ecosystem.

use crate::types::TranscodeOptions;
use napi::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

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

  // Create output file
  let mut output = BufWriter::new(File::create(output_path)
    .map_err(|e| Error::from_reason(format!("Failed to create output file: {}", e)))?);

  // Write EBML header for WebM
  write_webm_header(&mut output, final_width, final_height, final_frame_rate)?;

  // Write frames as SimpleBlocks in a Cluster
  let mut offset = 32; // Skip IVF header
  let mut frame_count = 0u32;

  // Start cluster
  write_cluster_start(&mut output, 0)?;

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

    // Write frame as SimpleBlock
    write_simpleblock(&mut output, frame_data, timestamp, frame_count)?;

    offset += 12 + frame_size;
    frame_count += 1;
  }

  output.flush()
    .map_err(|e| Error::from_reason(format!("Failed to flush output: {}", e)))?;

  Ok(())
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

  // Write IVF header
  write_ivf_header(&mut output_file, width, height, frame_rate)?;

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

  // Write WebM header
  write_webm_header(&mut output_file, width, height, frame_rate)?;

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

      // Write frame as SimpleBlock
      write_simpleblock(&mut output_file, yuv_data, frame_idx as u64, frame_idx)?;

      offset += frame_size;
      frame_idx += 1;
    } else {
      offset += 1;
    }
  }

  output_file.flush()
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
  _frame_rate: f64,
) -> Result<(), Error> {
  writer.write_all(b"DKIF")?;
  writer.write_all(&[0u8; 4])?; // Version
  writer.write_all(&[12u8, 0u8, 0u8, 0u8])?; // Header size
  writer.write_all(b"YV12")?; // FourCC (YV12 for uncompressed YUV420)
  writer.write_all(&width.to_le_bytes()[..2])?;
  writer.write_all(&height.to_le_bytes()[..2])?;
  writer.write_all(&[30u8, 0u8, 0u8, 0u8])?; // Timebase numerator
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

  // Tracks (0x1654ae6b)
  writer.write_all(&[0x16, 0x54, 0xae, 0x6b])?;
  writer.write_all(&[0x9e])?; // Size (30 bytes)

  // TrackEntry (0xae)
  writer.write_all(&[0xae])?;
  writer.write_all(&[0x8c])?; // Size (28 bytes)

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

  // CodecID (0x86) - V_RAWVIDEO for uncompressed
  writer.write_all(&[0x86])?;
  writer.write_all(&[0x8b])?;
  writer.write_all(b"V_RAWVIDEO")?;

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

fn write_cluster_start<W: Write>(
  writer: &mut W,
  timestamp: u64,
) -> Result<(), Error> {
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

  // Timestamp (signed, 2 bytes)
  writer.write_all(&[(timestamp & 0xFF) as u8, ((timestamp >> 8) & 0xFF) as u8])?;

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
    if token.starts_with('W') {
      width = token[1..].parse().unwrap_or(320);
    } else if token.starts_with('H') {
      height = token[1..].parse().unwrap_or(240);
    } else if token.starts_with('F') {
      let parts: Vec<&str> = token[1..].split(':').collect();
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

