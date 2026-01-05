//! Video filters module
//!
//! This module provides video processing filters.

use napi::Error;

/// Apply video filter with actual processing
pub fn apply_video_filter(frame_data: &[u8], filter_string: &str) -> Result<Vec<u8>, Error> {
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
) -> Result<Vec<u8>, Error> {
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
) -> Result<Vec<u8>, Error> {
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
    return Err(Error::from_reason(
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
fn apply_hflip_filter(frame_data: &[u8]) -> Result<Vec<u8>, Error> {
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
fn apply_vflip_filter(frame_data: &[u8]) -> Result<Vec<u8>, Error> {
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
fn apply_brightness_filter(frame_data: &[u8], adjustment: i32) -> Result<Vec<u8>, Error> {
  let mut adjusted_data = Vec::with_capacity(frame_data.len());

  for &byte in frame_data {
    let adjusted = (byte as i32 + adjustment).clamp(0, 255) as u8;
    adjusted_data.push(adjusted);
  }

  Ok(adjusted_data)
}

/// Apply contrast filter
fn apply_contrast_filter(frame_data: &[u8], contrast: f32) -> Result<Vec<u8>, Error> {
  let mut adjusted_data = Vec::with_capacity(frame_data.len());
  let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));

  for &byte in frame_data {
    let adjusted = (factor * (byte as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
    adjusted_data.push(adjusted);
  }

  Ok(adjusted_data)
}
