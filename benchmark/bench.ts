import { Bench } from 'tinybench'
import { GstKit } from '../index.js'

const b = new Bench({ time: 1000 })

// Initialize GStreamer once
const kit = new GstKit()
kit.setPipeline('videotestsrc num-buffers=1000 ! fakesink')
kit.play()

b.add('GstKit.getState()', () => {
  kit.getState()
})

b.add('GstKit.getPosition()', () => {
  kit.getPosition()
})

b.add('GstKit.getDuration()', () => {
  kit.getDuration()
})

b.add('GstKit.isInitialized()', () => {
  kit.isInitialized()
})

b.add('GstKit.getElements()', () => {
  kit.getElements()
})

await b.run()

console.table(b.table())

// Cleanup
kit.stop()
kit.cleanup()
