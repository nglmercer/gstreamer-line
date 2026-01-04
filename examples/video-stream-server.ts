/**
 * Video Stream Server
 *
 * This example demonstrates how to:
 * 1. Read a video file using GstKit
 * 2. Extract frames from video
 * 3. Stream frames via WebSocket to a web client
 *
 * Usage:
 *   bun run examples/video-stream-server.ts
 *
 * Then open http://localhost:8081 in your browser
 */

import { GstKit } from '../index.js';

// Configuration
const WS_PORT = 8081;
const VIDEO_PATH = './__test__/temp_output/test_red.avi'; // Default test video
const FRAME_RATE = 60; // Target frame rate
const VIDEO_DURATION = 10; // Video duration in seconds

// Video generation options
interface VideoFormat {
  name: string;
  extension: string;
  pipeline: string;
}

const VIDEO_FORMATS: VideoFormat[] = [
  {
    name: 'AVI (JPEG)',
    extension: 'avi',
    pipeline: `videotestsrc pattern=colors num-buffers=${VIDEO_DURATION * FRAME_RATE} ! video/x-raw,width=320,height=240,framerate=${FRAME_RATE}/1 ! jpegenc ! avimux ! filesink location="{path}"`
  },
  {
    name: 'WebM (VP8)',
    extension: 'webm',
    pipeline: `videotestsrc pattern=colors num-buffers=${VIDEO_DURATION * FRAME_RATE} ! video/x-raw,width=320,height=240,framerate=${FRAME_RATE}/1 ! vp8enc ! webmmux ! filesink location="{path}"`
  },
  {
    name: 'WebM (VP9)',
    extension: 'webm',
    pipeline: `videotestsrc pattern=colors num-buffers=${VIDEO_DURATION * FRAME_RATE} ! video/x-raw,width=320,height=240,framerate=${FRAME_RATE}/1 ! vp9enc ! webmmux ! filesink location="{path}"`
  },
  {
    name: 'MKV (VP8)',
    extension: 'mkv',
    pipeline: `videotestsrc pattern=colors num-buffers=${VIDEO_DURATION * FRAME_RATE} ! video/x-raw,width=320,height=240,framerate=${FRAME_RATE}/1 ! vp8enc ! matroskamux ! filesink location="{path}"`
  },
  {
    name: 'MKV (VP9)',
    extension: 'mkv',
    pipeline: `videotestsrc pattern=colors num-buffers=${VIDEO_DURATION * FRAME_RATE} ! video/x-raw,width=320,height=240,framerate=${FRAME_RATE}/1 ! vp9enc ! matroskamux ! filesink location="{path}"`
  },
];

class VideoStreamer {
  private gstKit: GstKit | null = null;
  private isPlaying: boolean = false;
  private frameInterval: NodeJS.Timeout | null = null;
  private wsClients: Set<any> = new Set();
  public currentVideoPath: string = VIDEO_PATH;
  private videoReady: boolean = false; // Track if video is ready for playback
  constructor() {
    this.startServer();
  }

  setCurrentVideoPath(videoPath: string) {
    this.currentVideoPath = videoPath;
  }

  setVideoReady(ready: boolean) {
    this.videoReady = ready;
  }

  private startServer() {
    const server = Bun.serve({
      port: WS_PORT,
      fetch: async (req) => {
        const url = new URL(req.url);
        
        if (url.pathname === '/') {
          return new Response(Bun.file('./examples/video-stream-client.html'), {
            headers: { 'Content-Type': 'text/html' },
          });
        }
        
        if (url.pathname === '/ws') {
          const upgraded = server.upgrade(req);
          if (!upgraded) {
            return new Response('WebSocket upgrade failed', { status: 400 });
          }
          return new Response();
        }
        
        return new Response('Not found', { status: 404 });
      },
      websocket: {
        open: (ws) => {
          console.log('✓ WebSocket client connected');
          this.wsClients.add(ws);

          // Send initial status
          ws.send(JSON.stringify({ type: 'status', data: 'connected' }));
          
          // Only start playback if video is ready
          if (this.videoReady) {
            this.stopVideo();
            setTimeout(() => {
              this.playVideo(this.currentVideoPath);
            }, 100);
          } else {
            console.log('⏳ Video not ready yet, waiting for generation to complete...');
          }
        },
        message: (ws, message) => {
          if (this.wsClients.has(ws))
          try {
            const data = JSON.parse(message.toString());
            
            if (data.type === 'play') {
              this.playVideo(data.videoPath || this.currentVideoPath);
            } else if (data.type === 'stop') {
              this.stopVideo();
            }
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
          }
        },
        close: (ws) => {
          console.log('✗ WebSocket client disconnected');
          this.wsClients.delete(ws);
        },
      },
    });

    console.log(`\n📡 WebSocket server started on ws://localhost:${WS_PORT}`);
    console.log(`🌐 Open http://localhost:${WS_PORT} in your browser\n`);
  }

  private broadcastFrame(frameBuffer: Buffer) {
    const base64 = frameBuffer.toString('base64');
    const message = JSON.stringify({ type: 'frame', data: base64 });

    for (const client of this.wsClients) {
      if (client.readyState === 1) { // WebSocket.OPEN
        try {
          client.send(message);
        } catch (error) {
          console.error('Failed to send frame to client:', error);
          this.wsClients.delete(client);
        }
      }
    }
  }

  async playVideo(videoPath: string) {
    if (this.isPlaying) {
      console.log('Video is already playing');
      return;
    }

    // Verify video file exists and is not empty before starting
    if (!fs.existsSync(videoPath)) {
      console.error(`❌ Video file not found: ${videoPath}`);
      return;
    }

    const stats = fs.statSync(videoPath);
    if (stats.size === 0) {
      console.error(`❌ Video file is empty: ${videoPath}`);
      return;
    }

    try {
      console.log(`\n🎬 Starting video playback: ${videoPath}`);
      this.gstKit = new GstKit();

      // Pipeline to decode video and extract frames as raw video data
      const pipeline = `
        filesrc location="${videoPath}" !
        avidemux ! jpegdec !
        queue max-size-buffers=10 max-size-time=0 max-size-bytes=0 !
        videoconvert ! video/x-raw,format=RGBA !
        appsink name=sink
      `;

      this.gstKit.setPipeline(pipeline);
      this.gstKit.play();

      // Wait a moment for pipeline to initialize
      await new Promise(resolve => setTimeout(resolve, 500));

      // Check pipeline state
      const state = this.gstKit.getState();
      console.log(`📊 Pipeline state: ${state}`);

      // Check if elements are present
      try {
        const elements = this.gstKit.getElements();
        console.log(`🔧 Pipeline elements: ${elements.join(', ')}`);
      } catch (error) {
        console.error('Failed to get elements:', error);
      }

      this.isPlaying = true;
      console.log('✓ GStreamer pipeline started');

      // Start frame extraction loop
      this.startFrameExtraction();

    } catch (error) {
      console.error('Failed to start video playback:', error);
      this.cleanup();
    }
  }

  private startFrameExtraction() {
    let frameCount = 0;
    const frameInterval = 1000 / FRAME_RATE; // ms between frames (~16.67ms for 60fps)

    this.frameInterval = setInterval(() => {
      if (!this.isPlaying || !this.gstKit) {
        return;
      }

      try {
        const frame = this.gstKit.pullSample('sink', 100); // 100ms timeout for better reliability

        if (frame) {
          frameCount++;
          this.broadcastFrame(frame);

          // Send progress updates every 60 frames (1 second at 60fps)
          if (frameCount % 60 === 0) {
            console.log(`📹 Streamed ${frameCount} frames (last frame size: ${frame.length} bytes)`);
          }
        } else {
          // Check for EOS (End of Stream) - if we keep getting null frames
          if (frameCount % 60 === 0) {
            console.log(`⚠️  No frame available for 60 consecutive attempts`);
          }

          // Stop if we've gone too long without frames
          if (frameCount >= VIDEO_DURATION * FRAME_RATE) {
            console.log(`\n✅ Video playback completed: ${frameCount} frames streamed`);
            this.stopVideo();
            return;
          }
        }
      } catch (error) {
        console.error('Error pulling frame:', error);
      }

    }, frameInterval);
  }

  stopVideo() {
    if (!this.isPlaying) {
      return;
    }

    console.log('\n⏹️  Stopping video playback');
    this.isPlaying = false;

    if (this.frameInterval) {
      clearInterval(this.frameInterval);
      this.frameInterval = null;
    }

    this.cleanup();
  }

  private cleanup() {
    if (this.gstKit) {
      this.gstKit.stop();
      this.gstKit.cleanup();
      this.gstKit = null;
    }
    console.log('✓ Cleanup complete');
  }
}

// Create and start video streamer
let streamer = new VideoStreamer();

// Check if video file exists, if not create a test video
import * as fs from 'node:fs';
import * as path from 'path';

async function createTestVideo(formatIndex: number = 0) {
  const format = VIDEO_FORMATS[formatIndex];
  const videoPath = path.join(path.dirname(VIDEO_PATH), `test_video.${format.extension}`);
  
  // Always regenerate the video to ensure correct duration and framerate
  if (fs.existsSync(videoPath)) {
    console.log(`\n🔄 Regenerating test video in ${format.name} format...`);
    fs.unlinkSync(videoPath);
  } else {
    console.log(`\n⚠️  Creating test video in ${format.name} format...`);
  }
  
  // Create test video using GstKit
  const kit = new GstKit();
  const pipeline = format.pipeline.replace('{path}', videoPath);
  
  kit.setPipeline(pipeline);
  kit.play();
  
  // Wait for video generation (10 seconds at 60fps)
  await new Promise<void>(resolve => setTimeout(resolve, (VIDEO_DURATION * 1000) + 1000));
  
  kit.stop();
  kit.cleanup();
  
  console.log(`✓ Test video created: ${format.name} (${VIDEO_DURATION}s @ ${FRAME_RATE}fps = ${VIDEO_DURATION * FRAME_RATE} frames)`);
  return videoPath;
}

// Parse command line arguments for format selection
const args = process.argv.slice(2);
let formatIndex = 0;

if (args.length > 0) {
  const formatArg = args.find(arg => arg.startsWith('--format=') || arg.startsWith('-f='));
  if (formatArg) {
    const index = parseInt(formatArg.split('=')[1]);
    if (!isNaN(index) && index >= 0 && index < VIDEO_FORMATS.length) {
      formatIndex = index;
    }
  }
}

// Create test video and start playing
(async () => {
  const videoPath = await createTestVideo(formatIndex);
  
  // Update the current video path so new connections use it
  streamer.setCurrentVideoPath(videoPath);
  
  // Mark video as ready - this allows WebSocket clients to start playback
  streamer.setVideoReady(true);
  console.log('✅ Video is ready for playback');
  
  // Start playing video after a short delay
  setTimeout(() => {
    streamer.playVideo(videoPath);
  }, 1000);
})();

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\n\n👋 Shutting down...');
  streamer.stopVideo();
  process.exit(0);
});

console.log(`🖥️  Server started on http://localhost:${WS_PORT}`);
