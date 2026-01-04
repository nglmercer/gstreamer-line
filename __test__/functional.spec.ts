
import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { GstKit } from '../index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';

const TEST_DIR = path.join(__dirname, 'temp_output');

describe('GstKit Functional Tests', () => {
    beforeAll(() => {
        if (!fs.existsSync(TEST_DIR)) {
            fs.mkdirSync(TEST_DIR);
        }
    });

    afterAll(() => {
        // Cleanup all test files
        if (fs.existsSync(TEST_DIR)) {
            const files = fs.readdirSync(TEST_DIR);
            for (const file of files) {
                fs.unlinkSync(path.join(TEST_DIR, file));
            }
            fs.rmdirSync(TEST_DIR);
        }
    });

    describe('Video Generation - Basic Patterns', () => {
        it('should generate a video with red pattern', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_red.avi');

            const pipeline = `
                videotestsrc pattern=red num-buffers=15 !
                video/x-raw,width=320,height=240,framerate=15/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate a video with snow pattern', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_snow.avi');

            const pipeline = `
                videotestsrc pattern=snow num-buffers=15 !
                video/x-raw,width=320,height=240,framerate=15/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate a video with color bars pattern', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_colors.avi');

            const pipeline = `
                videotestsrc pattern=colors num-buffers=15 !
                video/x-raw,width=320,height=240,framerate=15/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate a video with ball animation', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_ball.avi');

            const pipeline = `
                videotestsrc pattern=ball num-buffers=30 !
                video/x-raw,width=320,height=240,framerate=30/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });
    });

    describe('Video Generation - Different Formats', () => {
        it('should generate MP4 video with H.264 encoding', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_h264.mp4');

            // Try to use x264enc, fallback to jpegenc if not available
            let pipeline = `
                videotestsrc pattern=snow num-buffers=30 !
                video/x-raw,width=640,height=480,framerate=30/1 !
                x264enc ! mp4mux ! filesink location="${outputFile}"
            `;

            try {
                kit.setPipeline(pipeline);
                kit.play();
                await new Promise(r => setTimeout(r, 2000));
                kit.stop();

                expect(fs.existsSync(outputFile)).toBe(true);
                const stats = fs.statSync(outputFile);
                expect(stats.size).toBeGreaterThan(1000);
            } catch (error) {
                // If x264enc is not available, skip this test
                if (error instanceof Error && error.message.includes('no element "x264enc"')) {
                    console.log('Skipping H.264 test: x264enc not available');
                } else {
                    throw error;
                }
            }
        });

        it('should generate WebM video with VP8 encoding', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_vp8.webm');

            const pipeline = `
                videotestsrc pattern=colors num-buffers=30 !
                video/x-raw,width=640,height=480,framerate=30/1 !
                vp8enc ! webmmux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });
    });

    describe('Video Generation - With Audio', () => {
        it('should generate video with audio track', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_audio.avi');

            const pipeline = `
                videotestsrc pattern=colors num-buffers=30 !
                video/x-raw,width=320,height=240,framerate=30/1 !
                jpegenc ! queue ! avimux name=mux ! filesink location="${outputFile}"
                audiotestsrc wave=sine num-buffers=30 !
                audio/x-raw,rate=44100,channels=2 !
                lamemp3enc ! queue ! mux.
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate video with different audio waveforms', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_square.avi');

            const pipeline = `
                videotestsrc pattern=smpte num-buffers=30 !
                video/x-raw,width=320,height=240,framerate=30/1 !
                jpegenc ! queue ! avimux name=mux ! filesink location="${outputFile}"
                audiotestsrc wave=square num-buffers=30 !
                audio/x-raw,rate=44100,channels=2 !
                lamemp3enc ! queue ! mux.
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });
    });

    describe('Video Processing - Read and Decode', () => {
        const testFile = path.join(TEST_DIR, 'test_read.avi');

        beforeAll(async () => {
            // Create a test file first
            const kit = new GstKit();
            const pipeline = `
                videotestsrc pattern=red num-buffers=15 !
                video/x-raw,width=320,height=240,framerate=15/1 !
                jpegenc ! avimux ! filesink location="${testFile}"
            `;
            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();
        });

        it('should read and decode generated video file', async () => {
            const kit = new GstKit();
            const pipeline = `
                filesrc location="${testFile}" !
                avidemux ! jpegdec !
                videoconvert ! video/x-raw,format=RGBA !
                appsink name=sink
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 500));

            let framesCaptured = 0;
            for (let i = 0; i < 10; i++) {
                const frame = kit.pullSample("sink");
                if (frame) {
                    framesCaptured++;
                    expect(frame.length).toBe(320 * 240 * 4); // RGBA
                }
                await new Promise(r => setTimeout(r, 100));
            }

            kit.stop();
            expect(framesCaptured).toBeGreaterThan(0);
        });

        it('should get position while reading video', async () => {
            const kit = new GstKit();
            const pipeline = `
                filesrc location="${testFile}" !
                avidemux ! jpegdec !
                fakesink
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 500));

            const position = kit.getPosition();
            expect(position).toBeGreaterThanOrEqual(0);

            kit.stop();
        });

        it('should get duration of video file', async () => {
            const kit = new GstKit();
            const pipeline = `
                filesrc location="${testFile}" !
                avidemux ! jpegdec !
                fakesink
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 500));

            const duration = kit.getDuration();
            expect(duration).toBeGreaterThanOrEqual(-1);

            kit.stop();
        });
    });

    describe('Custom Video Generation - AppSrc', () => {
        it('should generate video from custom data using AppSrc', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_custom.avi');

            const pipeline = `
                appsrc name=source !
                video/x-raw,width=160,height=120,format=RGB,framerate=30/1 !
                videoconvert !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();

            // Generate and push 30 frames (1 second at 30fps)
            const frameSize = 160 * 120 * 3; // RGB
            for (let i = 0; i < 30; i++) {
                // Create a simple gradient pattern
                const buffer = Buffer.alloc(frameSize);
                for (let y = 0; y < 120; y++) {
                    for (let x = 0; x < 160; x++) {
                        const offset = (y * 160 + x) * 3;
                        const r = Math.floor((x / 160) * 255);
                        const g = Math.floor((y / 120) * 255);
                        const b = Math.floor(((x + y) / (160 + 120)) * 255);
                        buffer[offset] = r;
                        buffer[offset + 1] = g;
                        buffer[offset + 2] = b;
                    }
                }
                kit.pushSample('source', buffer);
                // Throttle to ~30fps
                await new Promise(r => setTimeout(r, 33));
            }

            kit.stop();
            kit.cleanup();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate animated pattern from custom data', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_animated_custom.avi');

            const pipeline = `
                appsrc name=source !
                video/x-raw,width=160,height=120,format=RGB,framerate=30/1 !
                videoconvert !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();

            // Generate 30 frames with moving pattern
            const frameSize = 160 * 120 * 3;
            for (let frame = 0; frame < 30; frame++) {
                const buffer = Buffer.alloc(frameSize);
                for (let y = 0; y < 120; y++) {
                    for (let x = 0; x < 160; x++) {
                        const offset = (y * 160 + x) * 3;
                        // Create a moving gradient
                        const offset_x = (x + frame * 2) % 160;
                        const offset_y = (y + frame) % 120;
                        buffer[offset] = Math.floor((offset_x / 160) * 255);
                        buffer[offset + 1] = Math.floor((offset_y / 120) * 255);
                        buffer[offset + 2] = Math.floor(((offset_x + offset_y) / (160 + 120)) * 255);
                    }
                }
                kit.pushSample('source', buffer);
                await new Promise(r => setTimeout(r, 33));
            }

            kit.stop();
            kit.cleanup();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });
    });

    describe('Video Effects and Processing', () => {
        it('should generate video with video balance effect', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_effect.avi');

            const pipeline = `
                videotestsrc pattern=colors num-buffers=30 !
                video/x-raw,width=320,height=240,framerate=30/1 !
                videobalance saturation=2.0 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate video with different resolution', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_resolution.avi');

            const pipeline = `
                videotestsrc pattern=snow num-buffers=30 !
                video/x-raw,width=800,height=600,framerate=30/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should generate video with different framerate', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_framerate.avi');

            const pipeline = `
                videotestsrc pattern=ball num-buffers=60 !
                video/x-raw,width=320,height=240,framerate=60/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 2000));
            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });
    });

    describe('Pipeline State Management During Generation', () => {
        it('should handle pause and resume during video generation', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_pause_resume.avi');

            const pipeline = `
                videotestsrc pattern=snow num-buffers=60 !
                video/x-raw,width=320,height=240,framerate=30/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();

            // Let it run for a bit
            await new Promise(r => setTimeout(r, 500));

            // Pause
            kit.pause();
            expect(kit.getState()).toBe('Paused');

            // Wait a bit
            await new Promise(r => setTimeout(r, 500));

            // Resume
            kit.play();
            expect(kit.getState()).toBe('Playing');

            // Let it finish
            await new Promise(r => setTimeout(r, 1500));

            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            expect(stats.size).toBeGreaterThan(1000);
        });

        it('should handle seeking during video playback', async () => {
            const kit = new GstKit();
            const outputFile = path.join(TEST_DIR, 'test_seek.avi');

            // Create a longer video for seeking (3 seconds at 30fps = 90 buffers)
            const pipeline = `
                videotestsrc pattern=ball num-buffers=90 !
                video/x-raw,width=320,height=240,framerate=30/1 !
                jpegenc ! avimux ! filesink location="${outputFile}"
            `;

            kit.setPipeline(pipeline);
            kit.play();
            await new Promise(r => setTimeout(r, 1000));

            // Seek to 1 second
            kit.seek(1_000_000_000); // 1 second in nanoseconds
            await new Promise(r => setTimeout(r, 500));

            kit.stop();

            expect(fs.existsSync(outputFile)).toBe(true);
            const stats = fs.statSync(outputFile);
            // File size may vary, just check it exists and has some content
            expect(stats.size).toBeGreaterThan(100);
        });
    });
});
