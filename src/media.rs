//! Media operations module
//!
//! This module provides a unified interface for media operations including
//! format detection, codec detection, and media processing.

use std::path::Path;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

// Import from sibling modules
use crate::format::{MediaFormat, format_name, format_long_name};

/// Media processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[napi(object)]
pub struct MediaProcessingResult {
    pub success: bool,
    pub message: String,
    pub format: Option<String>,
    pub codec: Option<String>,
}

/// Media processor interface
pub trait MediaProcessor {
    /// Detect format from file path
    fn detect_format(&self, path: &Path) -> Option<MediaFormat>;
    
    /// Get supported formats
    fn supported_formats(&self) -> Vec<String>;
    
    /// Get supported codecs
    fn supported_codecs(&self) -> Vec<String>;
}

/// Default media processor implementation
pub struct DefaultMediaProcessor;

impl MediaProcessor for DefaultMediaProcessor {
    fn detect_format(&self, path: &Path) -> Option<MediaFormat> {
        let format = crate::format::detect_format(path);
        if matches!(&format, MediaFormat::Unknown(_)) {
            None
        } else {
            Some(format)
        }
    }
    
    fn supported_formats(&self) -> Vec<String> {
        vec![
            "ivf".to_string(),
            "matroska".to_string(),
            "webm".to_string(),
            "y4m".to_string(),
        ]
    }
    
    fn supported_codecs(&self) -> Vec<String> {
        vec![
            "av1".to_string(),
            "vp8".to_string(),
            "vp9".to_string(),
            "h264".to_string(),
            "h265".to_string(),
            "opus".to_string(),
            "vorbis".to_string(),
            "pcm".to_string(),
        ]
    }
}

/// Create default media processor
pub fn create_processor() -> DefaultMediaProcessor {
    DefaultMediaProcessor
}

/// Validate media file
#[napi]
pub fn validate_media_file(path: String) -> MediaProcessingResult {
    let path_buf = Path::new(&path);
    
    if !path_buf.exists() {
        return MediaProcessingResult {
            success: false,
            message: format!("File not found: {}", path),
            format: None,
            codec: None,
        };
    }
    
    let processor = create_processor();
    let format = processor.detect_format(&path_buf);
    
    let format_name = match &format {
        Some(ref fmt) => Some(format_name(fmt).to_string()),
        None => None,
    };
    
    MediaProcessingResult {
        success: format.is_some(),
        message: match &format {
            Some(_) => "Format detected successfully".to_string(),
            None => "Unknown format".to_string(),
        },
        format: format_name,
        codec: None,
    }
}

/// Get media info summary
#[napi]
pub fn get_media_summary(path: String) -> String {
    let path_buf = Path::new(&path);
    let processor = create_processor();
    
    let format = processor.detect_format(&path_buf);
    let format_str = match format {
        Some(fmt) => format_long_name(&fmt),
        None => "Unknown format".to_string(),
    };
    
    format!(
        "File: {}\nFormat: {}\nSupported Formats: {:?}\nSupported Codecs: {:?}",
        path,
        format_str,
        processor.supported_formats(),
        processor.supported_codecs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_processor_detect_format() {
        let processor = create_processor();
        
        let ivf_path = Path::new("test.ivf");
        assert_eq!(processor.detect_format(&ivf_path), Some(MediaFormat::Ivf));
        
        let unknown_path = Path::new("test.unknown");
        assert_eq!(processor.detect_format(&unknown_path), None);
    }

    #[test]
    fn test_media_processor_supported_formats() {
        let processor = create_processor();
        let formats = processor.supported_formats();
        
        assert!(formats.contains(&"ivf".to_string()));
        assert!(formats.contains(&"matroska".to_string()));
        assert!(formats.contains(&"webm".to_string()));
        assert!(formats.contains(&"y4m".to_string()));
    }

    #[test]
    fn test_media_processor_supported_codecs() {
        let processor = create_processor();
        let codecs = processor.supported_codecs();
        
        assert!(codecs.contains(&"av1".to_string()));
        assert!(codecs.contains(&"vp9".to_string()));
        assert!(codecs.contains(&"h264".to_string()));
        assert!(codecs.contains(&"opus".to_string()));
    }

    #[test]
    fn test_validate_media_file_not_found() {
        let result = validate_media_file("/nonexistent/file.ivf".to_string());
        assert!(!result.success);
        assert!(result.message.contains("File not found"));
    }
}
