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
  writer.write_all(b"AV01")?; // FourCC (AV1)
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
