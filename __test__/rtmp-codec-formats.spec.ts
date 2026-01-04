/**
 * RTMP Codec Format Tests
 *
 * Tests for processing RTMP video streams with different codec formats
 * and output formats to find the optimal configuration.
 */

import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { GstKit } from '../index.js';

describe('RTMP Codec Format Tests', () => {
  let kit: GstKit;

  beforeAll(() => {
    kit = new GstKit();
  });

  afterAll(() => {
    if (kit.isInitialized()) {
      kit.stop();
      kit.cleanup();
    }
  });

  describe('H.264 to JPEG (Current Configuration)', () => {
    it('should process H.264 data and output JPEG frames', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      // Wait for pipeline to initialize
      await new Promise(resolve => setTimeout(resolve, 500));

      // Push some H.264 test data (simulated RTMP video data)
      // This is a minimal H.264 SPS/PPS/IDR frame
      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01, // NALU start code
        0x67, 0x42, 0x80, 0x0a, // SPS
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80, // PPS
        0x00, 0x00, 0x00, 0x01, // NALU start code
        0x65, 0x88, 0x80, 0x00, // IDR frame
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      // Try to pull a frame
      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 to PNG', () => {
    it('should process H.264 data and output PNG frames', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        pngenc compression-level=6 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 to Raw I420', () => {
    it('should process H.264 data and output raw I420 frames', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 to Raw RGBA', () => {
    it('should process H.264 data and output raw RGBA frames', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=RGBA,width=1280,height=720,framerate=30/1 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 with Different Resolutions', () => {
    it('should process H.264 at 640x480', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=640,height=480,framerate=30/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });

    it('should process H.264 at 1920x1080', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1920,height=1080,framerate=30/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 with Different Frame Rates', () => {
    it('should process H.264 at 15 fps', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=15/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });

    it('should process H.264 at 60 fps', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=60/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 with Buffer Sizes', () => {
    it('should handle max-buffers=1', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=1 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });

    it('should handle max-buffers=5', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=5 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('H.264 with Different JPEG Quality', () => {
    it('should process H.264 with JPEG quality=50', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        jpegenc quality=50 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });

    it('should process H.264 with JPEG quality=100', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        jpegenc quality=100 !
        appsink name=sink emit-signals=true sync=false max-buffers=2 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      kit.pushSample('src', h264Data);
      await new Promise(resolve => setTimeout(resolve, 200));

      const frame = kit.pullSample('sink', 200);

      kit.stop();
      kit.cleanup();

      expect(frame).not.toBeNull();
      expect(frame!.length).toBeGreaterThan(0);
    });
  });

  describe('Multiple H.264 Frames', () => {
    it('should process multiple H.264 frames', async () => {
      const pipeline = `
        appsrc name=src format=time is-live=true do-timestamp=true !
        h264parse config-interval=1 !
        avdec_h264 !
        videoconvert !
        video/x-raw,format=I420,width=1280,height=720,framerate=30/1 !
        jpegenc quality=90 !
        appsink name=sink emit-signals=true sync=false max-buffers=3 drop=true
      `;

      kit.setPipeline(pipeline);
      kit.play();

      await new Promise(resolve => setTimeout(resolve, 500));

      // Push 5 H.264 frames
      const h264Data = Buffer.from([
        0x00, 0x00, 0x00, 0x01,
        0x67, 0x42, 0x80, 0x0a,
        0xff, 0xe1, 0x00, 0x19, 0x67, 0x42, 0x80, 0x0a, 0xff, 0xe1, 0x00, 0x19,
        0x68, 0xce, 0x3c, 0x80,
        0x00, 0x00, 0x00, 0x01,
        0x65, 0x88, 0x80, 0x00,
      ]);

      for (let i = 0; i < 5; i++) {
        kit.pushSample('src', h264Data);
        await new Promise(resolve => setTimeout(resolve, 50));
      }

      // Try to pull frames
      let framesPulled = 0;
      for (let i = 0; i < 10; i++) {
        const frame = kit.pullSample('sink', 100);
        if (frame) {
          framesPulled++;
          expect(frame.length).toBeGreaterThan(0);
        }
        await new Promise(resolve => setTimeout(resolve, 50));
      }

      kit.stop();
      kit.cleanup();

      expect(framesPulled).toBeGreaterThan(0);
    });
  });
});
