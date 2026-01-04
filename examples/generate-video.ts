import { GstKit } from '../index.js'

// Example 1: Generate a simple test video with test pattern
async function generateSimpleTestVideo() {
  console.log('Generating simple test video...')
  const kit = new GstKit()

  // Create a pipeline that generates a test pattern and encodes it to MP4
  kit.setPipeline(
    'videotestsrc pattern=snow num-buffers=300 ! ' +
    'video/x-raw,width=640,height=480,framerate=30/1 ! ' +
    'x264enc tune=zerolatency ! ' +
    'mp4mux ! ' +
    'filesink location=test-video.mp4'
  )

  kit.play()

  // Wait for the pipeline to finish
  await new Promise(resolve => setTimeout(resolve, 11000))

  kit.stop()
  kit.cleanup()
  console.log('✓ Simple test video generated: test-video.mp4')
}

// Example 2: Generate a video with different test patterns
async function generatePatternVideo() {
  console.log('Generating video with color bars pattern...')
  const kit = new GstKit()

  kit.setPipeline(
    'videotestsrc pattern=colors num-buffers=300 ! ' +
    'video/x-raw,width=1280,height=720,framerate=30/1 ! ' +
    'vp8enc ! ' +
    'webmmux ! ' +
    'filesink location=test-pattern.webm'
  )

  kit.play()

  await new Promise(resolve => setTimeout(resolve, 11000))

  kit.stop()
  kit.cleanup()
  console.log('✓ Pattern video generated: test-pattern.webm')
}

// Example 3: Generate a video with audio
async function generateVideoWithAudio() {
  console.log('Generating video with audio...')
  const kit = new GstKit()

  kit.setPipeline(
    'videotestsrc pattern=ball num-buffers=300 ! ' +
    'video/x-raw,width=640,height=480,framerate=30/1 ! ' +
    'x264enc ! ' +
    'queue ! ' +
    'mp4mux name=mux ! ' +
    'filesink location=test-video-audio.mp4 ' +
    'audiotestsrc wave=sine num-buffers=300 ! ' +
    'audio/x-raw,rate=44100,channels=2 ! ' +
    'lamemp3enc ! ' +
    'queue ! ' +
    'mux.'
  )

  kit.play()

  await new Promise(resolve => setTimeout(resolve, 11000))

  kit.stop()
  kit.cleanup()
  console.log('✓ Video with audio generated: test-video-audio.mp4')
}

// Example 4: Generate a video from custom data using AppSrc
async function generateVideoFromCustomData() {
  console.log('Generating video from custom data...')
  const kit = new GstKit()

  kit.setPipeline(
    'appsrc name=source ! ' +
    'video/x-raw,width=320,height=240,format=RGB,framerate=30/1 ! ' +
    'videoconvert ! ' +
    'x264enc ! ' +
    'mp4mux ! ' +
    'filesink location=test-custom.mp4'
  )

  kit.play()

  // Generate and push 300 frames (10 seconds at 30fps)
  const frameSize = 320 * 240 * 3 // RGB
  for (let i = 0; i < 300; i++) {
    // Create a simple gradient pattern
    const buffer = Buffer.alloc(frameSize)
    for (let y = 0; y < 240; y++) {
      for (let x = 0; x < 320; x++) {
        const offset = (y * 320 + x) * 3
        const r = Math.floor((x / 320) * 255)
        const g = Math.floor((y / 240) * 255)
        const b = Math.floor(((x + y) / (320 + 240)) * 255)
        buffer[offset] = r
        buffer[offset + 1] = g
        buffer[offset + 2] = b
      }
    }
    kit.pushSample('source', buffer)
    // Throttle to ~30fps
    await new Promise(resolve => setTimeout(resolve, 33))
  }

  kit.stop()
  kit.cleanup()
  console.log('✓ Custom video generated: test-custom.mp4')
}

// Example 5: Generate an animated test video
async function generateAnimatedVideo() {
  console.log('Generating animated test video...')
  const kit = new GstKit()

  kit.setPipeline(
    'videotestsrc pattern=smpte num-buffers=300 ! ' +
    'video/x-raw,width=800,height=600,framerate=30/1 ! ' +
    'timeoverlay font-desc="Sans, 48" ! ' +
    'x264enc ! ' +
    'mp4mux ! ' +
    'filesink location/test-animated.mp4'
  )

  kit.play()

  await new Promise(resolve => setTimeout(resolve, 11000))

  kit.stop()
  kit.cleanup()
  console.log('✓ Animated video generated: test-animated.mp4')
}

// Run all examples
async function main() {
  try {
    await generateSimpleTestVideo()
    await generatePatternVideo()
    await generateVideoWithAudio()
    await generateVideoFromCustomData()
    await generateAnimatedVideo()
    console.log('\n✓ All test videos generated successfully!')
  } catch (error) {
    console.error('Error generating videos:', error)
    process.exit(1)
  }
}

main()
