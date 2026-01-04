
import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { GstKit } from '../index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';

const TEST_DIR = path.join(__dirname, 'temp_output');
const TEST_FILE = path.join(TEST_DIR, 'test_video.mkv');

describe('GstKit Functional Tests', () => {
    beforeAll(() => {
        if (!fs.existsSync(TEST_DIR)) {
            fs.mkdirSync(TEST_DIR);
        }
    });

    afterAll(() => {
        // Cleanup
        if (fs.existsSync(TEST_FILE)) {
            fs.unlinkSync(TEST_FILE);
        }
        if (fs.existsSync(TEST_DIR)) {
            fs.rmdirSync(TEST_DIR);
        }
    });

    it('should generate a video file using a pipeline', async () => {
        const kit = new GstKit();
        console.log("Generating test file at:", TEST_FILE);

        // Pipeline: Generate 1 second of video (red background) and save to AVI (MJPEG)
        // using plugins that are more likely to be present (gst-plugins-good)
        const pipeline = `
            videotestsrc pattern=red num-buffers=15 ! 
            video/x-raw,width=320,height=240,framerate=15/1 ! 
            jpegenc ! avimux ! filesink location="${TEST_FILE}"
        `;

        kit.setPipeline(pipeline);
        kit.play();

        // Wait for pipeline to finish
        await new Promise(r => setTimeout(r, 2000));

        kit.stop();

        // Check file existence and size
        expect(fs.existsSync(TEST_FILE)).toBe(true);
        const stats = fs.statSync(TEST_FILE);
        expect(stats.size).toBeGreaterThan(1000); // Should be some bytes
        console.log("Generated file size:", stats.size);
    });

    it('should process (read) the generated file', async () => {
        if (!fs.existsSync(TEST_FILE)) {
            console.warn("Skipping read test because file was not generated");
            return;
        }

        const kit = new GstKit();
        // Pipeline: Read file ! decode ! appsink
        const pipeline = `
            filesrc location="${TEST_FILE}" ! 
            avidemux ! jpegdec ! 
            videoconvert ! video/x-raw,format=RGBA ! 
            appsink name=sink
        `;

        kit.setPipeline(pipeline);
        kit.play();

        // Allow some time for pre-roll
        await new Promise(r => setTimeout(r, 500));

        // Poll for a few frames
        let framesCaptured = 0;
        for (let i = 0; i < 10; i++) {
            const frame = kit.pullSample("sink");
            if (frame) {
                framesCaptured++;
                expect(frame.length).toBe(320 * 240 * 4); // RGBA
            }
            await new Promise(r => setTimeout(r, 100)); // 10fps poll
        }

        kit.stop();
        expect(framesCaptured).toBeGreaterThan(0);
    });
});
