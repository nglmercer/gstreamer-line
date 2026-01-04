//! Media generation and validation tests
//!
//! This module tests the creation of media files in different formats
//! and validates them using the format/codec detection system.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use rust_av_kit::format::{MediaFormat, detect_format, format_name, format_long_name};
use rust_av_kit::codec::{MediaCodec, is_codec_supported, codec_name, codec_type, CodecType};
use rust_av_kit::media::{validate_media_file, MediaProcessingResult};
use rust_av_kit::transform_format;

/// Test configuration for generated media
struct TestMediaConfig {
    width: u32,
    height: u32,
    framerate: u32,
    duration_seconds: u32,
}

/// Generate a simple IVF (Indeo Video Format) file
///
/// IVF is a simple container format for VP8/VP9/AV1 bitstreams
fn generate_ivf_file(path: &Path, config: &TestMediaConfig) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    
    // IVF header (12 bytes)
    // Bytes 0-3: "DKIF" (signature)
    file.write_all(b"DKIF")?;
    
    // Bytes 4-7: Version (0)
    file.write_all(&[0u8; 4])?;
    
    // Bytes 8-11: Header length (12 bytes for version 0)
    file.write_all(&[12u8, 0u8, 0u8, 0u8])?;
    
    // Bytes 12-15: FourCC (VP90 = 0x30395090, VP80 = 0x30385090, AV01 = 0x30313050)
    file.write_all(&[0x30, 0x39, 0x50, 0x90])?; // VP90
    
    // Bytes 16-19: Width
    file.write_all(&config.width.to_le_bytes())?; // Little-endian
    
    // Bytes 20-23: Height
    file.write_all(&config.height.to_le_bytes())?;
    
    // Bytes 24-27: Framerate (timebase numerator)
    file.write_all(&config.framerate.to_le_bytes())?;
    
    // Bytes 28-31: Timebase denominator
    file.write_all(&[1u8; 4])?;
    
    // Bytes 32-35: Number of frames
    let num_frames = config.framerate * config.duration_seconds;
    file.write_all(&num_frames.to_le_bytes())?;
    
    // Write some dummy frame data (simplified)
    let frame_size = config.width * config.height * 3; // RGB
    let total_frame_data_size = num_frames as usize * frame_size as usize;
    let dummy_frame_data = vec![0u8; total_frame_data_size];
    file.write_all(&dummy_frame_data)?;
    
    Ok(())
}

/// Generate a simple Y4M (YUV4MPEG2) file
///
/// Y4M is an uncompressed video format
fn generate_y4m_file(path: &Path, config: &TestMediaConfig) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    
    // Y4M header
    let header = format!(
        "YUV4MPEG2 {} {} {} {}\n",
        config.width, config.height, config.framerate, config.duration_seconds
    );
    file.write_all(header.as_bytes())?;
    
    // Write dummy YUV420 frame data
    let y_size = (config.width * config.height) as usize;
    let uv_size = y_size / 2;
    let total_size = y_size + uv_size;
    let dummy_data = vec![128u8; total_size];
    file.write_all(&dummy_data)?;
    
    Ok(())
}

/// Generate a simple Matroska/WebM file
///
/// Matroska is a container format (simplified EBML structure)
fn generate_matroska_file(path: &Path, config: &TestMediaConfig) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    
    // Simplified EBML header for Matroska/WebM
    let header = b"\x1a\x45\xdf\xa3";
    file.write_all(header)?;
    
    // Write dummy data
    let frame_size = config.width * config.height * 3;
    let num_frames = config.framerate * config.duration_seconds;
    let total_size = num_frames as usize * frame_size as usize;
    let dummy_data = vec![0u8; total_size];
    file.write_all(&dummy_data)?;
    
    Ok(())
}

/// Generate a test video file based on format
fn generate_test_video(path: &Path, format: &MediaFormat, config: &TestMediaConfig) -> std::io::Result<()> {
    match format {
        MediaFormat::Ivf => generate_ivf_file(path, config),
        MediaFormat::Matroska => generate_matroska_file(path, config),
        MediaFormat::Y4m => generate_y4m_file(path, config),
        _ => {
            // For unknown formats, create a simple text file
            let mut file = fs::File::create(path)?;
            let content = format!(
                "Unknown format test video\nWidth: {}\nHeight: {}\nFramerate: {}\nDuration: {}s\n",
                config.width, config.height, config.framerate, config.duration_seconds
            );
            file.write_all(content.as_bytes())?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_generate_ivf_file() {
        let test_dir = PathBuf::from("temp_frames/test_videos");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 320,
            height: 240,
            framerate: 30,
            duration_seconds: 1,
        };
        
        let ivf_path = test_dir.join("test.ivf");
        assert!(generate_test_video(&ivf_path, &MediaFormat::Ivf, &config).is_ok());
        
        // Validate format detection
        let detected = detect_format(&ivf_path);
        assert_eq!(detected, MediaFormat::Ivf);
        assert_eq!(format_name(&MediaFormat::Ivf), "ivf");
        
        // Clean up
        fs::remove_file(&ivf_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_generate_y4m_file() {
        let test_dir = PathBuf::from("temp_frames/test_videos");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 640,
            height: 480,
            framerate: 25,
            duration_seconds: 2,
        };
        
        let y4m_path = test_dir.join("test.y4m");
        assert!(generate_test_video(&y4m_path, &MediaFormat::Y4m, &config).is_ok());
        
        // Validate format detection
        let detected = detect_format(&y4m_path);
        assert_eq!(detected, MediaFormat::Y4m);
        assert_eq!(format_name(&MediaFormat::Y4m), "y4m");
        
        // Clean up
        fs::remove_file(&y4m_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_generate_matroska_file() {
        let test_dir = PathBuf::from("temp_frames/test_videos");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 1280,
            height: 720,
            framerate: 30,
            duration_seconds: 3,
        };
        
        let mkv_path = test_dir.join("test.mkv");
        assert!(generate_test_video(&mkv_path, &MediaFormat::Matroska, &config).is_ok());
        
        // Validate format detection
        let detected = detect_format(&mkv_path);
        assert_eq!(detected, MediaFormat::Matroska);
        assert_eq!(format_name(&MediaFormat::Matroska), "matroska");
        
        // Clean up
        fs::remove_file(&mkv_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_validate_existing_file() {
        let test_dir = PathBuf::from("temp_frames/test_validate_existing");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 320,
            height: 240,
            framerate: 30,
            duration_seconds: 1,
        };
        
        let test_path = test_dir.join("test.ivf");
        generate_test_video(&test_path, &MediaFormat::Ivf, &config).unwrap();
        
        // Validate file
        let result = validate_media_file(test_path.to_string_lossy().to_string());
        assert!(result.success);
        assert!(result.format.is_some());
        assert_eq!(result.format, Some("ivf".to_string()));
        
        // Clean up
        fs::remove_file(&test_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_validate_nonexistent_file() {
        let result = validate_media_file("/nonexistent/test.ivf".to_string());
        assert!(!result.success);
        assert!(result.message.contains("File not found"));
        assert!(result.format.is_none());
    }

    #[test]
    fn test_codec_support_validation() {
        // Test supported codecs
        assert!(is_codec_supported("av1"));
        assert!(is_codec_supported("vp9"));
        assert!(is_codec_supported("h264"));
        assert!(is_codec_supported("opus"));
        assert!(is_codec_supported("pcm"));
        
        // Test unsupported codecs
        assert!(!is_codec_supported("unknown"));
        assert!(!is_codec_supported("unsupported"));
    }

    #[test]
    fn test_codec_names() {
        assert_eq!(codec_name(&MediaCodec::Av1), "av1");
        assert_eq!(codec_name(&MediaCodec::Vp9), "vp9");
        assert_eq!(codec_name(&MediaCodec::H264), "h264");
        assert_eq!(codec_name(&MediaCodec::Opus), "opus");
        assert_eq!(codec_name(&MediaCodec::Vorbis), "vorbis");
    }

    #[test]
    fn test_codec_types() {
        assert_eq!(codec_type(&MediaCodec::Av1), CodecType::Video);
        assert_eq!(codec_type(&MediaCodec::Vp8), CodecType::Video);
        assert_eq!(codec_type(&MediaCodec::H265), CodecType::Video);
        assert_eq!(codec_type(&MediaCodec::Opus), CodecType::Audio);
        assert_eq!(codec_type(&MediaCodec::Vorbis), CodecType::Audio);
        assert_eq!(codec_type(&MediaCodec::Pcm), CodecType::Audio);
    }

    #[test]
    fn test_format_long_names() {
        assert_eq!(format_long_name(&MediaFormat::Ivf), "Indeo Video Format (IVF)");
        assert_eq!(format_long_name(&MediaFormat::Matroska), "Matroska/WebM container");
        assert_eq!(format_long_name(&MediaFormat::Y4m), "YUV4MPEG2 uncompressed video");
    }

    #[test]
    fn test_multiple_formats_generation() {
        let test_dir = PathBuf::from("temp_frames/test_videos");
        fs::create_dir_all(&test_dir).ok();
        
        let configs = vec![
            TestMediaConfig {
                width: 320,
                height: 240,
                framerate: 30,
                duration_seconds: 1,
            },
            TestMediaConfig {
                width: 640,
                height: 480,
                framerate: 25,
                duration_seconds: 2,
            },
            TestMediaConfig {
                width: 1280,
                height: 720,
                framerate: 30,
                duration_seconds: 3,
            },
        ];
        
        let formats = vec![
            (MediaFormat::Ivf, "test1.ivf"),
            (MediaFormat::Y4m, "test2.y4m"),
            (MediaFormat::Matroska, "test3.mkv"),
        ];
        
        for (format, filename) in &formats {
            let path = test_dir.join(filename);
            assert!(generate_test_video(&path, &format, &configs[0]).is_ok());
        }
        
        // Validate all files
        for (format, filename) in &formats {
            let path = test_dir.join(filename);
            let detected = detect_format(&path);
            assert_eq!(detected, *format);
        }
        
        // Clean up
        for (_, filename) in &formats {
            let path = test_dir.join(filename);
            fs::remove_file(&path).ok();
        }
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_media_processing_result() {
        let result = MediaProcessingResult {
            success: true,
            message: "Test successful".to_string(),
            format: Some("ivf".to_string()),
            codec: Some("av1".to_string()),
        };
        
        assert!(result.success);
        assert_eq!(result.message, "Test successful");
        assert_eq!(result.format, Some("ivf".to_string()));
        assert_eq!(result.codec, Some("av1".to_string()));
    }

    #[test]
    fn test_transform_ivf_to_matroska() {
        let test_dir = PathBuf::from("temp_frames/test_transform_ivf_to_matroska");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 320,
            height: 240,
            framerate: 30,
            duration_seconds: 1,
        };
        
        let input_path = test_dir.join("test_input.ivf");
        let output_path = test_dir.join("test_output.mkv");
        
        generate_test_video(&input_path, &MediaFormat::Ivf, &config).unwrap();
        
        assert!(transform_format(
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string()
        ).is_ok());
        
        assert!(output_path.exists());
        let detected = detect_format(&output_path);
        assert_eq!(detected, MediaFormat::Matroska);
        
        fs::remove_file(&input_path).ok();
        fs::remove_file(&output_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_transform_matroska_to_ivf() {
        let test_dir = PathBuf::from("temp_frames/test_transform_matroska_to_ivf");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 320,
            height: 240,
            framerate: 30,
            duration_seconds: 1,
        };
        
        let input_path = test_dir.join("test_input.mkv");
        let output_path = test_dir.join("test_output.ivf");
        
        generate_test_video(&input_path, &MediaFormat::Matroska, &config).unwrap();
        
        assert!(transform_format(
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string()
        ).is_ok());
        
        assert!(output_path.exists());
        let detected = detect_format(&output_path);
        assert_eq!(detected, MediaFormat::Ivf);
        
        fs::remove_file(&input_path).ok();
        fs::remove_file(&output_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_transform_y4m_to_ivf() {
        let test_dir = PathBuf::from("temp_frames/test_transform_y4m_to_ivf");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 640,
            height: 480,
            framerate: 25,
            duration_seconds: 1,
        };
        
        let input_path = test_dir.join("test_input.y4m");
        let output_path = test_dir.join("test_output.ivf");
        
        generate_test_video(&input_path, &MediaFormat::Y4m, &config).unwrap();
        
        assert!(transform_format(
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string()
        ).is_ok());
        
        assert!(output_path.exists());
        let detected = detect_format(&output_path);
        assert_eq!(detected, MediaFormat::Ivf);
        
        fs::remove_file(&input_path).ok();
        fs::remove_file(&output_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_transform_ivf_to_y4m() {
        let test_dir = PathBuf::from("temp_frames/test_transform_ivf_to_y4m");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 640,
            height: 480,
            framerate: 25,
            duration_seconds: 1,
        };
        
        let input_path = test_dir.join("test_input.ivf");
        let output_path = test_dir.join("test_output.y4m");
        
        generate_test_video(&input_path, &MediaFormat::Ivf, &config).unwrap();
        
        assert!(transform_format(
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string()
        ).is_ok());
        
        assert!(output_path.exists());
        let detected = detect_format(&output_path);
        assert_eq!(detected, MediaFormat::Y4m);
        
        fs::remove_file(&input_path).ok();
        fs::remove_file(&output_path).ok();
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_transform_nonexistent_file() {
        let test_dir = PathBuf::from("temp_frames/test_transform_nonexistent");
        fs::create_dir_all(&test_dir).ok();
        
        let output_path = test_dir.join("test_output.mkv");
        
        let result = transform_format(
            "/nonexistent/input.ivf".to_string(),
            output_path.to_string_lossy().to_string()
        );
        
        assert!(result.is_err());
        
        fs::remove_dir(&test_dir).ok();
    }

    #[test]
    fn test_transform_unsupported_conversion() {
        let test_dir = PathBuf::from("temp_frames/test_transform_unsupported");
        fs::create_dir_all(&test_dir).ok();
        
        let config = TestMediaConfig {
            width: 320,
            height: 240,
            framerate: 30,
            duration_seconds: 1,
        };
        
        let input_path = test_dir.join("test_input.mkv");
        let output_path = test_dir.join("test_output.y4m");
        
        generate_test_video(&input_path, &MediaFormat::Matroska, &config).unwrap();
        
        let result = transform_format(
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string()
        );
        
        assert!(result.is_err());
        
        fs::remove_file(&input_path).ok();
        fs::remove_dir(&test_dir).ok();
    }
}
