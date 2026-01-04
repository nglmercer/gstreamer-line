/**
 * OBS Stream Simulation Test
 * 
 * This file simulates an OBS RTMP stream by generating H.264 encoded video
 * and pushing it through the StreamProcessor to verify frame generation works correctly.
 */

import { GstKit } from '../../index.js';
import { StreamProcessor } from './server.js';
import { EventEmitter } from 'events';

// Configuration
const WIDTH = 320;
const HEIGHT = 240;
const FPS = 30;
const NUM_FRAMES = 30; // Generate 1 second of video

class OBSSimulator {
  private gstKit: GstKit | null = null;
  private frameCount: number = 0;
  private eventEmitter: EventEmitter;

  constructor() {
    this.eventEmitter = new EventEmitter();
  }

  /**
   * Initialize the GStreamer pipeline to generate H.264 encoded frames
   */
  async initialize(): Promise<void> {
    console.log('🎬 Initializing OBS Simulator...');
    
    this.gstKit = new GstKit();

    // Pipeline to generate test video and encode to H.264
    // Output in AVCC format (stream-format=avc) as RTMP expects
    const pipelineString = `
      videotestsrc pattern=ball num-buffers=${NUM_FRAMES} !
      video/x-raw,width=${WIDTH},height=${HEIGHT},framerate=${FPS}/1 !
      videoconvert !
      x264enc tune=zerolatency speed-preset=superfast !
      h264parse !
      video/x-h264,stream-format=avc,alignment=au !
      appsink name=source emit-signals=true sync=false max-buffers=1 drop=true
    `;

    this.gstKit.setPipeline(pipelineString);
    this.gstKit.play();
    
    // Wait for pipeline to start
    await new Promise(resolve => setTimeout(resolve, 500));
    
    console.log('✅ OBS Simulator initialized');
  }

  /**
   * Get next H.264 encoded frame from the pipeline
   */
  async getNextH264Frame(): Promise<Buffer | null> {
    if (!this.gstKit) {
      return null;
    }

    const frame = this.gstKit.pullSample('source', 100);
    
    if (frame) {
      this.frameCount++;
      this.eventEmitter.emit('frame', frame);
      
      if (this.frameCount % 10 === 0) {
        console.log(`📹 Generated ${this.frameCount} H.264 frames`);
      }
    }
    
    return frame;
  }

  /**
   * Wrap H.264 data in RTMP video packet format
   * RTMP video packet structure:
   * Byte 0: Frame type (0x17 for keyframe, 0x27 for interframe)
   * Byte 1: AVC packet type (0=sequence header, 1=NALU, 2=end of sequence)
   * Bytes 2-3: Composition time (24-bit, little-endian)
   * Bytes 4+: H.264 data
   */
  wrapAsRTMPVideo(h264Data: Buffer, isKeyframe: boolean): Buffer {
    const frameType = isKeyframe ? 0x17 : 0x27;
    const avcPacketType = isKeyframe ? 0 : 1; // 0 for sequence header, 1 for NALU
    const compositionTime = 0; // 24-bit, little-endian

    const header = Buffer.alloc(4);
    header[0] = frameType;
    header[1] = avcPacketType;
    header[2] = compositionTime & 0xFF;
    header[3] = (compositionTime >> 8) & 0xFF;

    return Buffer.concat([header, h264Data]);
  }

  /**
   * Generate all frames and emit them
   */
  async generateAllFrames(): Promise<void> {
    console.log(`🎬 Generating ${NUM_FRAMES} H.264 frames...`);
    
    for (let i = 0; i < NUM_FRAMES; i++) {
      const h264Frame = await this.getNextH264Frame();
      
      if (h264Frame) {
        // First frame is a keyframe (sequence header)
        const isKeyframe = i === 0;
        const rtmpPacket = this.wrapAsRTMPVideo(h264Frame, isKeyframe);
        
        this.eventEmitter.emit('rtmp-video', rtmpPacket);
      }
      
      // Throttle to target framerate
      await new Promise(resolve => setTimeout(resolve, 1000 / FPS));
    }
    
    console.log('✅ Finished generating all frames');
  }

  /**
   * Get the event emitter for frame events
   */
  getEmitter(): EventEmitter {
    return this.eventEmitter;
  }

  /**
   * Cleanup resources
   */
  cleanup(): void {
    if (this.gstKit) {
      this.gstKit.stop();
      this.gstKit.cleanup();
      this.gstKit = null;
    }
  }
}

/**
 * Test the StreamProcessor with simulated OBS stream
 */
async function testStreamProcessor(): Promise<void> {
  console.log('\n🧪 Testing StreamProcessor with simulated OBS stream...\n');
  
  const obsSimulator = new OBSSimulator();
  const streamProcessor = new StreamProcessor();
  
  try {
    // Initialize OBS simulator
    await obsSimulator.initialize();
    
    // Initialize stream processor
    await streamProcessor.initialize('test-stream');
    
    // Listen for frame events
    obsSimulator.getEmitter().on('frame', (h264Data: Buffer) => {
      console.log(`📥 OBS Simulator generated H.264 frame: ${h264Data.length} bytes`);
    });
    
    // Listen for RTMP video packets and send to stream processor
    obsSimulator.getEmitter().on('rtmp-video', (rtmpPacket: Buffer) => {
      console.log(`📤 Sending RTMP video packet to StreamProcessor: ${rtmpPacket.length} bytes`);
      streamProcessor.processVideoBuffer(rtmpPacket);
    });
    
    // Generate all frames
    await obsSimulator.generateAllFrames();
    
    // Wait for frames to be processed
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Check results
    const status = streamProcessor.getPipelineStatus();
    console.log(`\n📊 StreamProcessor Status:`);
    console.log(`   Initialized: ${status.initialized}`);
    console.log(`   Frames processed: ${status.frameCount}`);
    
    if (status.frameCount > 0) {
      console.log(`\n✅ SUCCESS! StreamProcessor processed ${status.frameCount} frames`);
    } else {
      console.log(`\n❌ FAILED! No frames were processed`);
    }
    
  } catch (error) {
    console.error('❌ Error during test:', error);
  } finally {
    // Cleanup
    obsSimulator.cleanup();
    streamProcessor.cleanup();
  }
}

/**
 * Test direct GstKit frame extraction
 */
async function testDirectFrameExtraction(): Promise<void> {
  console.log('\n🧪 Testing direct GstKit frame extraction...\n');
  
  const kit = new GstKit();
  
  try {
    // Simple pipeline: generate video -> decode -> RGBA
    const pipelineString = `
      videotestsrc pattern=ball num-buffers=${NUM_FRAMES} !
      video/x-raw,width=${WIDTH},height=${HEIGHT},framerate=${FPS}/1 !
      videoconvert !
      video/x-raw,format=RGBA,width=${WIDTH},height=${HEIGHT},framerate=${FPS}/1 !
      appsink name=sink emit-signals=true sync=false max-buffers=10 drop=true
    `;

    kit.setPipeline(pipelineString);
    kit.play();
    
    // Wait for pipeline to start
    await new Promise(resolve => setTimeout(resolve, 500));
    
    console.log('📹 Pulling frames from pipeline...');
    
    let frameCount = 0;
    let consecutiveNullFrames = 0;
    const maxNullFrames = 10;
    
    while (consecutiveNullFrames < maxNullFrames && frameCount < NUM_FRAMES) {
      const frame = kit.pullSample('sink', 100);
      
      if (frame) {
        frameCount++;
        consecutiveNullFrames = 0;
        
        console.log(`✅ Pulled frame ${frameCount}, size: ${frame.length} bytes`);
        
        if (frameCount % 10 === 0) {
          console.log(`📹 Progress: ${frameCount}/${NUM_FRAMES} frames`);
        }
      } else {
        consecutiveNullFrames++;
        console.log(`⏳ Waiting for frame... (${consecutiveNullFrames}/${maxNullFrames})`);
      }
      
      // Throttle to target framerate
      await new Promise(resolve => setTimeout(resolve, 1000 / FPS));
    }
    
    console.log(`\n📊 Results:`);
    console.log(`   Total frames pulled: ${frameCount}`);
    console.log(`   Expected frames: ${NUM_FRAMES}`);
    
    if (frameCount >= NUM_FRAMES * 0.9) { // Allow 10% tolerance
      console.log(`✅ SUCCESS! Frame extraction working correctly`);
    } else {
      console.log(`⚠️  WARNING: Fewer frames than expected`);
    }
    
  } catch (error) {
    console.error('❌ Error during direct test:', error);
  } finally {
    kit.stop();
    kit.cleanup();
  }
}

// Main test runner
async function runTests() {
  console.log('╔══════════════════════════════════════════════════════════╗');
  console.log('║  OBS Stream Simulation Test Suite                              ║');
  console.log('╚══════════════════════════════════════════════════════════╝\n');
  
  // Test 1: Direct frame extraction
  console.log('Test 1: Direct GstKit Frame Extraction');
  console.log('─────────────────────────────────────────────────────────────────\n');
  await testDirectFrameExtraction();
  
  console.log('\n\n');
  
  // Test 2: StreamProcessor with simulated OBS
  console.log('Test 2: StreamProcessor with Simulated OBS Stream');
  console.log('─────────────────────────────────────────────────────────────────\n');
  await testStreamProcessor();
  
  console.log('\n\n');
  console.log('╔════════════════════════════════════════════════════════════╗');
  console.log('║  All tests completed                                            ║');
  console.log('╚══════════════════════════════════════════════════════════╝\n');
}

// Run tests
runTests().catch(error => {
  console.error('❌ Fatal error:', error);
  process.exit(1);
});
