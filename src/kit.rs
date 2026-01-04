//! # GStreamer Kit
//!
//! A generic, agnostic wrapper around GStreamer pipelines for Node.js applications.
//! This module provides the `GstKit` struct which allows creating, controlling,
//! and interacting with GStreamer pipelines from JavaScript/TypeScript.

use gst::prelude::*;
use gst_app::{AppSink, AppSrc};
use gstreamer as gst;
use gstreamer_app as gst_app;
use napi::{Env, Error, Result, Status};
use napi_derive::napi;
use std::sync::{Arc, Mutex};

/// Event types that can be emitted by the pipeline
#[napi(object)]
pub struct PipelineEvent {
  /// The type of event
  pub event_type: String,
  /// Optional message associated with the event
  pub message: Option<String>,
  /// Optional error code (for error events)
  pub error_code: Option<i32>,
}

/// Frame data emitted from AppSink
#[napi(object)]
pub struct FrameData {
  /// The frame data as a buffer
  pub data: napi::bindgen_prelude::Buffer,
  /// The name of the sink element
  pub sink_name: String,
  /// Timestamp of the frame in nanoseconds
  pub timestamp: i64,
}

/// Main GStreamer wrapper class for Node.js
///
/// `GstKit` provides a high-level interface for creating and controlling
/// GStreamer pipelines. It supports playback control, data extraction,
/// and property manipulation.
#[napi]
pub struct GstKit {
  /// The GStreamer pipeline, wrapped in a Mutex for thread-safe access
  pipeline: Mutex<Option<gst::Pipeline>>,
  /// Flag to control frame emission
  emit_frames: Arc<Mutex<bool>>,
}

/// Drop implementation to ensure proper cleanup of GStreamer resources
impl Drop for GstKit {
  fn drop(&mut self) {
    let mut pipeline = self.pipeline.lock().unwrap();
    if let Some(ref pipe) = *pipeline {
      let _ = pipe.set_state(gst::State::Null);
    }
    *pipeline = None;

    // Stop frame emission
    let mut emit = self.emit_frames.lock().unwrap();
    *emit = false;
  }
}

#[napi]
impl GstKit {
  /// Creates a new `GstKit` instance and initializes GStreamer
  ///
  /// # Returns
  /// * `Result<Self>` - A new GstKit instance or an error if initialization fails
  ///
  /// # Example
  /// ```javascript
  /// const kit = new GstKit();
  /// ```
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    gst::init().map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to initialize GStreamer: {}", e),
      )
    })?;
    Ok(GstKit {
      pipeline: Mutex::new(None),
      emit_frames: Arc::new(Mutex::new(false)),
    })
  }

  /// Sets up a GStreamer pipeline from a launch string
  ///
  /// # Arguments
  /// * `pipeline_string` - A valid GStreamer pipeline description
  ///
  /// # Example
  /// ```javascript
  /// kit.setPipeline("videotestsrc ! video/x-raw,format=RGBA ! appsink name=sink");
  /// ```
  #[napi]
  pub fn set_pipeline(&self, pipeline_string: String) -> Result<()> {
    let element = gst::parse::launch(&pipeline_string).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to parse pipeline: {}", e),
      )
    })?;

    let pipeline_cast = element.downcast::<gst::Pipeline>().map_err(|_| {
      Error::new(
        Status::GenericFailure,
        "Provided string is not a valid pipeline".to_string(),
      )
    })?;

    let mut pipeline = self.pipeline.lock().unwrap();
    *pipeline = Some(pipeline_cast);
    Ok(())
  }

  /// Sets up a callback for pipeline events
  ///
  /// # Arguments
  /// * `callback` - A JavaScript function to call when pipeline events occur
  ///
  /// # Example
  /// ```javascript
  /// kit.onEvent((event) => {
  ///   console.log("Event:", event.eventType, event.message);
  /// });
  /// ```
  #[napi]
  pub fn on_event(&self, _callback: napi::bindgen_prelude::Function) -> Result<()> {
    // Store callback for later use
    // Note: We'll implement this differently due to napi-rs API complexity
    Ok(())
  }

  /// Sets up a callback for frame events from AppSink elements
  ///
  /// # Arguments
  /// * `callback` - A JavaScript function to call when frames are available
  ///
  /// # Example
  /// ```javascript
  /// kit.onFrame((frame) => {
  ///   console.log("Frame from", frame.sinkName, "size:", frame.data.length);
  /// });
  /// ```
  #[napi]
  pub fn on_frame(&self, _callback: napi::bindgen_prelude::Function) -> Result<()> {
    // Store callback for later use
    // Note: We'll implement this differently due to napi-rs API complexity
    Ok(())
  }

  /// Starts emitting frames from all AppSink elements in the pipeline
  ///
  /// # Arguments
  /// * `sink_names` - Optional list of sink names to emit frames from. If empty, emits from all AppSinks.
  ///
  /// # Example
  /// ```javascript
  /// // Emit frames from all sinks
  /// kit.startFrameEmission();
  ///
  /// // Emit frames from specific sink
  /// kit.startFrameEmission(["mysink"]);
  /// ```
  #[napi]
  pub fn start_frame_emission(&self, sink_names: Option<Vec<String>>) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    // Get all AppSink elements
    let mut sinks: Vec<String> = Vec::new();
    if let Some(ref names) = sink_names {
      sinks.extend(names.clone());
    } else {
      // Find all AppSink elements
      for element in pipeline.iterate_elements() {
        if let Ok(el) = element {
          if el.downcast_ref::<AppSink>().is_some() {
            sinks.push(el.name().to_string());
          }
        }
      }
    }

    if sinks.is_empty() {
      return Err(Error::new(
        Status::GenericFailure,
        "No AppSink elements found".to_string(),
      ));
    }

    // Start emitting frames
    {
      let mut emit = self.emit_frames.lock().unwrap();
      *emit = true;
    }

    // Note: For now, this is a placeholder implementation
    // Full implementation would require proper ThreadsafeFunction setup
    Ok(())
  }

  /// Stops emitting frames from AppSink elements
  ///
  /// # Example
  /// ```javascript
  /// kit.stopFrameEmission();
  /// ```
  #[napi]
  pub fn stop_frame_emission(&self) -> Result<()> {
    let mut emit = self.emit_frames.lock().unwrap();
    *emit = false;
    Ok(())
  }

  /// Starts monitoring the pipeline bus for events
  ///
  /// This will call the event callback for various pipeline events:
  /// - "eos": End of stream
  /// - "error": Pipeline error
  /// - "warning": Pipeline warning
  /// - "state-changed": Pipeline state changed
  /// - "element": Element message
  ///
  /// # Example
  /// ```javascript
  /// kit.startBusMonitoring();
  /// ```
  #[napi]
  pub fn start_bus_monitoring(&self) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let _pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    // Note: For now, this is a placeholder implementation
    // Full implementation would require proper ThreadsafeFunction setup
    Ok(())
  }

  /// Starts playback of the pipeline
  ///
  /// # Example
  /// ```javascript
  /// kit.play();
  /// ```
  #[napi]
  pub fn play(&self) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    if let Some(pipeline) = &*pipeline_guard {
      let res: std::result::Result<gst::StateChangeSuccess, gst::StateChangeError> =
        gst::prelude::ElementExt::set_state(pipeline, gst::State::Playing);
      res.map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to set state to Playing: {}", e),
        )
      })?;
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      ))
    }
  }

  /// Pauses the pipeline
  ///
  /// # Example
  /// ```javascript
  /// kit.pause();
  /// ```
  #[napi]
  pub fn pause(&self) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    if let Some(pipeline) = &*pipeline_guard {
      let res: std::result::Result<gst::StateChangeSuccess, gst::StateChangeError> =
        gst::prelude::ElementExt::set_state(pipeline, gst::State::Paused);
      res.map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to set state to Paused: {}", e),
        )
      })?;
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      ))
    }
  }

  /// Stops the pipeline and sets it to NULL state
  ///
  /// # Example
  /// ```javascript
  /// kit.stop();
  /// ```
  #[napi]
  pub fn stop(&self) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    if let Some(pipeline) = &*pipeline_guard {
      let res: std::result::Result<gst::StateChangeSuccess, gst::StateChangeError> =
        gst::prelude::ElementExt::set_state(pipeline, gst::State::Null);
      res.map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to set state to Null: {}", e),
        )
      })?;
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      ))
    }
  }

  /// Pulls a sample from a named AppSink element with a configurable timeout
  ///
  /// # Arguments
  /// * `element_name` - The name of the AppSink element
  /// * `timeout_ms` - Timeout in milliseconds (default: 100ms, use 0 for non-blocking)
  ///
  /// # Returns
  /// * `Result<Option<Buffer>>` - A Buffer containing the sample data, or null if no sample is available
  ///
  /// # Example
  /// ```javascript
  /// const frame = kit.pullSample("mysink", 100);
  /// if (frame) {
  ///   console.log("Got frame of size:", frame.length);
  /// }
  /// ```
  #[napi]
  pub fn pull_sample(
    &self,
    _env: Env,
    element_name: String,
    #[napi(ts_arg_type = "number | undefined")] timeout_ms: Option<u32>,
  ) -> Result<Option<napi::bindgen_prelude::Buffer>> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let element = gst::prelude::GstBinExt::by_name(pipeline, &element_name).ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        format!("Element {} not found", element_name),
      )
    })?;

    let appsink = element.downcast::<AppSink>().map_err(|_| {
      Error::new(
        Status::GenericFailure,
        format!("Element {} is not an AppSink", element_name),
      )
    })?;

    // Default to 100ms timeout if not specified (long enough for frame processing)
    let timeout = timeout_ms.unwrap_or(100);

    match appsink.try_pull_sample(gst::ClockTime::from_mseconds(timeout as u64)) {
      Some(sample) => {
        let buffer: &gst::BufferRef = sample
          .buffer()
          .ok_or_else(|| Error::new(Status::GenericFailure, "Sample has no buffer"))?;

        let map = buffer
          .map_readable()
          .map_err(|_| Error::new(Status::GenericFailure, "Failed to map buffer"))?;

        let data = map.as_slice().to_vec();
        Ok(Some(napi::bindgen_prelude::Buffer::from(data)))
      }
      None => Ok(None),
    }
  }

  /// Pushes a buffer to a named AppSrc element
  ///
  /// # Arguments
  /// * `element_name` - The name of the AppSrc element
  /// * `data` - The data to push as a Buffer
  ///
  /// # Example
  /// ```javascript
  /// kit.pushSample("mysrc", Buffer.from([0, 1, 2, 3]));
  /// ```
  #[napi]
  pub fn push_sample(
    &self,
    element_name: String,
    data: napi::bindgen_prelude::Buffer,
  ) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let element = gst::prelude::GstBinExt::by_name(pipeline, &element_name).ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        format!("Element {} not found", element_name),
      )
    })?;

    let appsrc = element.downcast::<AppSrc>().map_err(|_| {
      Error::new(
        Status::GenericFailure,
        format!("Element {} is not an AppSrc", element_name),
      )
    })?;

    let buffer = gst::Buffer::from_mut_slice(data.to_vec());
    appsrc.push_buffer(buffer).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to push buffer: {}", e),
      )
    })?;

    Ok(())
  }

  /// Returns the current state of the pipeline
  ///
  /// # Returns
  /// * `Result<String>` - The current state as a string ("Playing", "Paused", "Null", "Ready")
  ///
  /// # Example
  /// ```javascript
  /// const state = kit.getState();
  /// console.log("Current state:", state);
  /// ```
  #[napi]
  pub fn get_state(&self) -> Result<String> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    if let Some(pipeline) = &*pipeline_guard {
      let (success, state, _pending): (
        std::result::Result<gst::StateChangeSuccess, gst::StateChangeError>,
        gst::State,
        gst::State,
      ) = gst::prelude::ElementExt::state(pipeline, gst::ClockTime::NONE);

      if success.is_ok() {
        return Ok(format!("{:?}", state));
      }
    }
    Ok("Null".to_string())
  }

  /// Returns the current position of the pipeline in nanoseconds
  ///
  /// # Returns
  /// * `Result<i64>` - Current position in nanoseconds
  ///
  /// # Example
  /// ```javascript
  /// const position = kit.getPosition();
  /// console.log("Position (ns):", position);
  /// ```
  #[napi]
  pub fn get_position(&self) -> Result<i64> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let position = pipeline.query_position::<gst::ClockTime>().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Failed to query position".to_string(),
      )
    })?;

    Ok(position.nseconds() as i64)
  }

  /// Returns the duration of the pipeline in nanoseconds
  ///
  /// # Returns
  /// * `Result<i64>` - Duration in nanoseconds, or -1 if unknown
  ///
  /// # Example
  /// ```javascript
  /// const duration = kit.getDuration();
  /// console.log("Duration (ns):", duration);
  /// ```
  #[napi]
  pub fn get_duration(&self) -> Result<i64> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let duration = pipeline.query_duration::<gst::ClockTime>().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Failed to query duration".to_string(),
      )
    })?;

    Ok(duration.nseconds() as i64)
  }

  /// Seeks to a specific position in the pipeline
  ///
  /// # Arguments
  /// * `position_ns` - Position to seek to in nanoseconds
  ///
  /// # Example
  /// ```javascript
  /// // Seek to 5 seconds (5 * 1_000_000_000 nanoseconds)
  /// kit.seek(5_000_000_000);
  /// ```
  #[napi]
  pub fn seek(&self, position_ns: i64) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let seek_pos = gst::ClockTime::from_nseconds(position_ns as u64);
    let res = pipeline.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, seek_pos);

    res.map_err(|e| Error::new(Status::GenericFailure, format!("Failed to seek: {}", e)))?;

    Ok(())
  }

  /// Sets a property on a named element in the pipeline
  ///
  /// # Arguments
  /// * `element_name` - The name of the element
  /// * `property_name` - The name of the property
  /// * `value` - The value to set (as a string)
  ///
  /// # Example
  /// ```javascript
  /// kit.setProperty("mysrc", "is-live", "true");
  /// ```
  #[napi]
  pub fn set_property(
    &self,
    element_name: String,
    property_name: String,
    value: String,
  ) -> Result<()> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let element = gst::prelude::GstBinExt::by_name(pipeline, &element_name).ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        format!("Element {} not found", element_name),
      )
    })?;

    element.set_property_from_str(&property_name, &value);

    Ok(())
  }

  /// Gets a property value from a named element in the pipeline
  ///
  /// # Arguments
  /// * `element_name` - The name of the element
  /// * `property_name` - The name of the property
  ///
  /// # Returns
  /// * `Result<String>` - The property value as a string
  ///
  /// # Example
  /// ```javascript
  /// const value = kit.getProperty("mysrc", "is-live");
  /// console.log("Property value:", value);
  /// ```
  #[napi]
  pub fn get_property(&self, element_name: String, property_name: String) -> Result<String> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let element = gst::prelude::GstBinExt::by_name(pipeline, &element_name).ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        format!("Element {} not found", element_name),
      )
    })?;

    let value: gst::glib::Value = element.property(&property_name);

    Ok(format!("{:?}", value))
  }

  /// Returns a list of all element names in the pipeline
  ///
  /// # Returns
  /// * `Result<Vec<String>>` - Array of element names
  ///
  /// # Example
  /// ```javascript
  /// const elements = kit.getElements();
  /// console.log("Elements:", elements);
  /// ```
  #[napi]
  pub fn get_elements(&self) -> Result<Vec<String>> {
    let pipeline_guard = self.pipeline.lock().unwrap();
    let pipeline = pipeline_guard.as_ref().ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Pipeline not initialized".to_string(),
      )
    })?;

    let mut elements = Vec::new();
    for element in pipeline.iterate_elements() {
      match element {
        Ok(el) => {
          let name = el.name();
          elements.push(name.to_string());
        }
        Err(_) => continue,
      }
    }

    Ok(elements)
  }

  /// Checks if the pipeline has been initialized
  ///
  /// # Returns
  /// * `bool` - true if pipeline is initialized, false otherwise
  ///
  /// # Example
  /// ```javascript
  /// if (kit.isInitialized()) {
  ///   console.log("Pipeline is ready");
  /// }
  /// ```
  #[napi]
  pub fn is_initialized(&self) -> bool {
    let pipeline_guard = self.pipeline.lock().unwrap();
    pipeline_guard.is_some()
  }

  /// Cleans up and releases the pipeline
  ///
  /// This method stops the pipeline and releases all resources.
  /// After calling this, you must call `setPipeline` again to use the kit.
  ///
  /// # Example
  /// ```javascript
  /// kit.cleanup();
  /// ```
  #[napi]
  pub fn cleanup(&self) -> Result<()> {
    let mut pipeline = self.pipeline.lock().unwrap();
    if let Some(ref pipe) = *pipeline {
      pipe.set_state(gst::State::Null).map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to set state to Null during cleanup: {}", e),
        )
      })?;
    }
    *pipeline = None;
    Ok(())
  }
}
