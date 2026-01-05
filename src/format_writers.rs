//! Format writers module
//!
//! This module provides functionality for writing media files in various formats.

use std::io::Write;

/// Write IVF header
pub fn write_ivf_header<W: Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  _frame_rate: f64,
) -> Result<(), napi::Error> {
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

/// Write IVF frame
pub fn write_ivf_frame<W: Write>(
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
pub fn write_matroska_header<W: Write>(
  writer: &mut W,
  width: i32,
  height: i32,
  frame_rate: f64,
) -> Result<(), napi::Error> {
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

  // Duration (0x4489) - placeholder for now
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

/// Write Matroska SimpleBlock
pub fn write_matroska_simpleblock<W: Write>(
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

/// Write Matroska Cluster header
pub fn write_matroska_cluster_start<W: Write>(
  writer: &mut W,
  timestamp: u64,
) -> Result<(), napi::Error> {
  // Cluster element ID (0x1F43B675)
  writer.write_all(&[0x1F, 0x43, 0xB6, 0x75])?;
  // Unknown size (will be updated later if needed)
  writer.write_all(&[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])?;
  
  // Timecode (0xE7)
  writer.write_all(&[0xE7])?;
  writer.write_all(&[0x83])?;
  writer.write_all(&(timestamp as u32).to_le_bytes())?;

  Ok(())
}

/// Write Matroska trailer
pub fn write_matroska_trailer<W: Write>(writer: &mut W) -> Result<(), napi::Error> {
  // Void element to pad
  writer.write_all(&[0xEC])?;
  writer.write_all(&[0x01])?;
  writer.write_all(&[0x00])?;
  writer.flush()?;

  Ok(())
}

/// Write Y4M header
pub fn write_y4m_header<W: Write>(
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
pub fn write_y4m_frame<W: Write>(
  writer: &mut W,
  frame_data: &[u8],
  _frame_number: u32,
) -> Result<(), napi::Error> {
  writer.write_all(b"FRAME\n")?;
  writer.write_all(frame_data)?;

  Ok(())
}
