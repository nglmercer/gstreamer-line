//! # GStreamer Kit - Native Node.js Module
//!
//! This library provides a high-level, agnostic wrapper around GStreamer pipelines
//! for Node.js applications. It allows creating, controlling, and interacting with
//! GStreamer pipelines from JavaScript/TypeScript code.
//!
//! ## Features
//!
//! - Pipeline creation from launch strings
//! - Playback control (play, pause, stop)
//! - Data extraction from AppSink elements
//! - Data injection via AppSrc elements
//! - Seeking and position/duration queries
//! - Property manipulation on pipeline elements
//! - Pipeline inspection and state management
//!
//! ## Example
//!
//! ```typescript
//! import { GstKit } from './index.js';
//!
//! const kit = new GstKit();
//! kit.setPipeline("videotestsrc ! video/x-raw,format=RGBA ! appsink name=sink");
//! kit.play();
//!
//! setInterval(() => {
//!   const frame = kit.pullSample("sink");
//!   if (frame) {
//!     console.log("Got frame of size:", frame.length);
//!   }
//! }, 33);
//! ```

#![deny(clippy::all)]

pub mod kit;

// Re-export the main struct for convenience
pub use kit::GstKit;
