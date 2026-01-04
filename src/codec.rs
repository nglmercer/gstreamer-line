//! Codec detection and handling module
//!
//! This module provides functionality for working with media codecs.

/// Supported media codecs
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCodec {
    /// AV1 (AOMedia Video 1)
    Av1,
    /// VP8 (VP8 Video Codec)
    Vp8,
    /// VP9 (VP9 Video Codec)
    Vp9,
    /// H.264/AVC (Advanced Video Coding)
    H264,
    /// H.265/HEVC (High Efficiency Video Coding)
    H265,
    /// Opus audio codec
    Opus,
    /// Vorbis audio codec
    Vorbis,
    /// PCM (Pulse Code Modulation) audio
    Pcm,
    /// Unknown codec
    Unknown(String),
}

/// Codec type (video, audio, subtitle, data, unknown)
#[derive(Debug, Clone, PartialEq)]
pub enum CodecType {
    Video,
    Audio,
    Subtitle,
    Data,
    Unknown,
}

/// Get codec name
pub fn codec_name(codec: &MediaCodec) -> String {
    match codec {
        MediaCodec::Av1 => "av1".to_string(),
        MediaCodec::Vp8 => "vp8".to_string(),
        MediaCodec::Vp9 => "vp9".to_string(),
        MediaCodec::H264 => "h264".to_string(),
        MediaCodec::H265 => "h265".to_string(),
        MediaCodec::Opus => "opus".to_string(),
        MediaCodec::Vorbis => "vorbis".to_string(),
        MediaCodec::Pcm => "pcm".to_string(),
        MediaCodec::Unknown(name) => name.clone(),
    }
}

/// Get codec type
pub fn codec_type(codec: &MediaCodec) -> CodecType {
    match codec {
        MediaCodec::Av1 | MediaCodec::Vp8 | MediaCodec::Vp9 | 
        MediaCodec::H264 | MediaCodec::H265 => CodecType::Video,
        MediaCodec::Opus | MediaCodec::Vorbis | MediaCodec::Pcm => CodecType::Audio,
        MediaCodec::Unknown(_) => CodecType::Unknown,
    }
}

/// Check if codec is supported
pub fn is_codec_supported(codec: &str) -> bool {
    matches!(codec.to_lowercase().as_str(), 
        "av1" | "vp8" | "vp9" | "h264" | "h265" | 
        "opus" | "vorbis" | "pcm"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_name() {
        assert_eq!(codec_name(&MediaCodec::Av1), "av1");
        assert_eq!(codec_name(&MediaCodec::H264), "h264");
        assert_eq!(codec_name(&MediaCodec::Opus), "opus");
    }

    #[test]
    fn test_codec_type() {
        assert_eq!(codec_type(&MediaCodec::Av1), CodecType::Video);
        assert_eq!(codec_type(&MediaCodec::Opus), CodecType::Audio);
        assert_eq!(codec_type(&MediaCodec::Unknown("test".to_string())), CodecType::Unknown);
    }

    #[test]
    fn test_is_codec_supported() {
        assert!(is_codec_supported("av1"));
        assert!(is_codec_supported("H264"));
        assert!(is_codec_supported("OPUS"));
        assert!(!is_codec_supported("unknown"));
    }
}
