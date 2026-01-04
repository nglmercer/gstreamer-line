/**
 * Simple H.264 Encoding Test
 * 
 * This test verifies that H.264 encoding works correctly
 */

import { GstKit } from '../../index.js';
import { isCodecAvailable } from '../../__test__/setup.js';

async function testEncoding(): Promise<void> {
  console.log('🧪 Testing H.264 Encoding...\n');
  
  // Check if H.264 encoder is available
  const h264Available = await isCodecAvailable('h264');
  if (!h264Available) {
    console.error('❌ H.264 encoder NOT available!');
    console.error('Please install GStreamer plugins: gstreamer1.0-plugins-good or gstreamer1.0-libav');
    process.exit(1);
  }
  
  console.log('✅ H.264 encoder is available\n');
  
  const kit = new GstKit();
  
  try {
    // Simple pipeline: generate 10 frames and encode to H.264
    // Increased max-buffers to 10 to allow more frames to accumulate
    // Removed drop=true to prevent buffers from being dropped
    const pipelineString = `
      videotestsrc pattern=ball num-buffers=10 !
      video/x-raw,width=320,height=240,framerate=30/1 !
      videoconvert !
      x264enc tune=zerolatency speed-preset=superfast !
      h264parse !
      appsink name=sink emit-signals=true sync=false max-buffers=10
    `;

    console.log('Setting up pipeline...');
    kit.setPipeline(pipelineString);
    
    console.log('Starting pipeline...');
    kit.play();
    
    // Wait for pipeline to start
    await new Promise(resolve => setTimeout(resolve, 500));
    
    const state = kit.getState();
    console.log(`Pipeline state: ${state}`);
    
    if (state !== 'Playing') {
      console.error('❌ Pipeline not in Playing state!');
      throw new Error(`Pipeline state is ${state}, expected Playing`);
    }
    
    console.log('\n📹 Pulling H.264 encoded frames...\n');
    
    let frameCount = 0;
    let consecutiveNullFrames = 0;
    const maxNullFrames = 5;
    
    while (consecutiveNullFrames < maxNullFrames && frameCount < 10) {
      const frame = kit.pullSample('sink', 200);
      
      if (frame) {
        frameCount++;
        consecutiveNullFrames = 0;
        
        console.log(`✅ Frame ${frameCount}: ${frame.length} bytes`);
      } else {
        consecutiveNullFrames++;
        console.log(`⏳ Waiting for frame... (${consecutiveNullFrames}/${maxNullFrames})`);
      }
      
      // Throttle to target framerate
      await new Promise(resolve => setTimeout(resolve, 33));
    }
    
    console.log(`\n📊 Results:`);
    console.log(`   Total frames: ${frameCount}`);
    console.log(`   Expected: 10`);
    
    if (frameCount >= 8) { // Allow 20% tolerance
      console.log(`\n✅ SUCCESS! H.264 encoding is working correctly`);
    } else {
      console.log(`\n❌ FAILED! H.264 encoding is not working`);
    }
    
  } catch (error) {
    console.error('❌ Error during test:', error);
  } finally {
    kit.stop();
    kit.cleanup();
  }
}

// Run test
testEncoding().catch(error => {
  console.error('❌ Fatal error:', error);
  process.exit(1);
});
