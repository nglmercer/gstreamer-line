import { GstKit } from '../index.js';

async function main() {
    console.log("Initializing GstKit...");
    const kit = new GstKit();

    // Example 1: Simple Test Pattern
    console.log("Setting up videotestsrc pipeline...");
    // Note: We explicitly force RGBA so the buffer data makes sense for typical UI rendering
    kit.setPipeline("videotestsrc pattern=ball ! video/x-raw,width=320,height=240,format=RGBA ! appsink name=test_sink");

    console.log("Playing...");
    kit.play();

    // Poll frames
    const interval = setInterval(() => {
        try {
            const frame = kit.pullSample("test_sink");
            if (frame) {
                console.log(`[Frame] Size: ${frame.length} bytes (Expected: ${320 * 240 * 4})`);
            }
        } catch (e) {
            console.error("Error pulling sample:", e);
        }
    }, 100);

    // Stop after 5 seconds
    setTimeout(() => {
        clearInterval(interval);
        console.log("Stopping...");
        kit.stop();
    }, 5000);
}

main().catch(console.error);
