//! Type definitions for Rust-AV Kit
//!
//! This module contains all data structures used throughout the library.

use napi_derive::napi;
use serde::{Deserialize, Serialize};

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
