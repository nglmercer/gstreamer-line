use napi::{Error, Result, Status, Env};
use napi_derive::napi;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gst::prelude::*;
use gst_app::{AppSink, AppSinkCallbacks};
use std::sync::Mutex;

#[napi]
pub struct GstKit {
    pipeline: Mutex<Option<gst::Pipeline>>,
}

#[napi]
impl GstKit {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        gst::init().map_err(|e| Error::new(Status::GenericFailure, format!("Failed to initialize GStreamer: {}", e)))?;
        Ok(GstKit { pipeline: Mutex::new(None) })
    }

    #[napi]
    pub fn set_pipeline(&self, pipeline_string: String) -> Result<()> {
        let element = gst::parse::launch(&pipeline_string)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to parse pipeline: {}", e)))?;
        
        let pipeline_cast = element.downcast::<gst::Pipeline>()
            .map_err(|_| Error::new(Status::GenericFailure, "Provided string is not a valid pipeline".to_string()))?;

        let mut pipeline = self.pipeline.lock().unwrap();
        *pipeline = Some(pipeline_cast);
        Ok(())
    }

    #[napi]
    pub fn play(&self) -> Result<()> {
        let pipeline_guard = self.pipeline.lock().unwrap();
        if let Some(pipeline) = &*pipeline_guard {
            let res: std::result::Result<gst::StateChangeSuccess, gst::StateChangeError> = 
                gst::prelude::ElementExt::set_state(pipeline, gst::State::Playing);
            res.map_err(|e| Error::new(Status::GenericFailure, format!("Failed to set state to Playing: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Pipeline not initialized".to_string()))
        }
    }

    #[napi]
    pub fn pause(&self) -> Result<()> {
        let pipeline_guard = self.pipeline.lock().unwrap();
        if let Some(pipeline) = &*pipeline_guard {
            let res: std::result::Result<gst::StateChangeSuccess, gst::StateChangeError> = 
                gst::prelude::ElementExt::set_state(pipeline, gst::State::Paused);
            res.map_err(|e| Error::new(Status::GenericFailure, format!("Failed to set state to Paused: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Pipeline not initialized".to_string()))
        }
    }

    #[napi]
    pub fn stop(&self) -> Result<()> {
        let pipeline_guard = self.pipeline.lock().unwrap();
        if let Some(pipeline) = &*pipeline_guard {
            let res: std::result::Result<gst::StateChangeSuccess, gst::StateChangeError> = 
                gst::prelude::ElementExt::set_state(pipeline, gst::State::Null);
            res.map_err(|e| Error::new(Status::GenericFailure, format!("Failed to set state to Null: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Pipeline not initialized".to_string()))
        }
    }

    #[napi]
    pub fn pull_sample(&self, _env: Env, element_name: String) -> Result<Option<napi::bindgen_prelude::Buffer>> {
        let pipeline_guard = self.pipeline.lock().unwrap();
        let pipeline = pipeline_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Pipeline not initialized".to_string()))?;

        let element = 
            gst::prelude::GstBinExt::by_name(pipeline, &element_name)
            .ok_or_else(|| Error::new(Status::GenericFailure, format!("Element {} not found", element_name)))?;

        let appsink = element.downcast::<AppSink>()
            .map_err(|_| Error::new(Status::GenericFailure, format!("Element {} is not an AppSink", element_name)))?;

        match appsink.try_pull_sample(gst::ClockTime::from_mseconds(5)) {
            Some(sample) => {
                let buffer: &gst::BufferRef = sample.buffer().ok_or_else(|| Error::new(Status::GenericFailure, "Sample has no buffer"))?;
                
                let map = buffer.map_readable().map_err(|_| Error::new(Status::GenericFailure, "Failed to map buffer"))?;
                
                let data = map.as_slice().to_vec();
                Ok(Some(napi::bindgen_prelude::Buffer::from(data)))
            }
            None => Ok(None),
        }
    }

    #[napi]
    pub fn get_state(&self) -> Result<String> {
        let pipeline_guard = self.pipeline.lock().unwrap();
        if let Some(pipeline) = &*pipeline_guard {
            let (success, state, _pending): (std::result::Result<gst::StateChangeSuccess, gst::StateChangeError>, gst::State, gst::State) 
                = gst::prelude::ElementExt::state(pipeline, gst::ClockTime::NONE);
            
            if success.is_ok() {
               return Ok(format!("{:?}", state));
            }
        }
        Ok("Null".to_string())
    }
}
