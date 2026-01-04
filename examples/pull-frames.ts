/**
 * Example: Pulling Processed Frames from RTMP Stream
 *
 * This example demonstrates how to:
 * 1. Start the RTMP server
 * 2. Pull processed frames from the GStreamer pipeline
 * 3. Send frames to clients (placeholder for WebSocket/other transport)
 */

import { RTMPServer, getStreamProcessor } from "./server.js";

// Start the RTMP server
const server = new RTMPServer(1935);
const processor = getStreamProcessor();

// Frame counter
let frameCount = 0;
let totalBytes = 0;

// Pull processed frames in a loop
setInterval(() => {
  const frame = processor.pullProcessedFrame();

  if (frame) {
    frameCount++;
    totalBytes += frame.length;

    // Every 30 frames (approx 1 second at 30 FPS), log stats
    if (frameCount % 30 === 0) {
      console.log(`[Frame Stats] Count: ${frameCount}, Total: ${(totalBytes / 1024 / 1024).toFixed(2)} MB`);
    }

    // TODO: Send frame to clients via WebSocket or other transport
    // Example:
    // webSocket.send(frame);
  }
}, 33); // ~30 FPS (1000ms / 30 ≈ 33ms)

console.log("Frame puller started - waiting for stream...");
console.log("Stream to this server using OBS:");
console.log("  Server: rtmp://localhost:1935/live");
console.log("  Stream Key: any_key");
