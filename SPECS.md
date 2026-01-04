# GStreamer Kit (Native Module)

The `GstKit` class provides a generic, agnostic wrapper around GStreamer pipelines for Node.js applications.

## API Documentation

### Class: `GstKit`

#### `constructor()`
Initializes the GStreamer library. Must be called before using other methods.

**Returns:** A new `GstKit` instance

**Example:**
```typescript
const kit = new GstKit();
```

---

#### `setPipeline(pipelineString: string): void`
Constructs a GStreamer pipeline from a launch string.

**Parameters:**
- `pipelineString`: Valid GStreamer pipeline description (e.g., `videotestsrc ! appsink name=sink`)

**Throws:** Error if the pipeline string is invalid

**Example:**
```typescript
kit.setPipeline("videotestsrc ! video/x-raw,format=RGBA ! appsink name=mysink");
```

---

#### `play(): void`
Sets the pipeline state to `PLAYING`.

**Throws:** Error if pipeline is not initialized or state change fails

**Example:**
```typescript
kit.play();
```

---

#### `pause(): void`
Sets the pipeline state to `PAUSED`.

**Throws:** Error if pipeline is not initialized or state change fails

**Example:**
```typescript
kit.pause();
```

---

#### `stop(): void`
Sets the pipeline state to `NULL` (stopped).

**Throws:** Error if pipeline is not initialized or state change fails

**Example:**
```typescript
kit.stop();
```

---

#### `pullSample(elementName: string): Buffer | null`
Attempts to pull a sample buffer from a named `appsink` element in the pipeline.

**Parameters:**
- `elementName`: The name of the `appsink` element (e.g., "sink")

**Returns:** A `Buffer` containing the sample data, or `null` if no sample is available

**Throws:** Error if pipeline is not initialized, element not found, or element is not an AppSink

**Example:**
```typescript
const frame = kit.pullSample("mysink");
if (frame) {
  console.log("Got frame of size:", frame.length);
}
```

---

#### `pushSample(elementName: string, data: Buffer): void`
Pushes a buffer to a named `appsrc` element in the pipeline.

**Parameters:**
- `elementName`: The name of the `appsrc` element (e.g., "mysrc")
- `data`: The data to push as a Buffer

**Throws:** Error if pipeline is not initialized, element not found, or element is not an AppSrc

**Example:**
```typescript
kit.pushSample("mysrc", Buffer.from([0, 1, 2, 3]));
```

---

#### `getState(): string`
Returns the current state of the pipeline as a string.

**Returns:** Current state: "Playing", "Paused", "Null", or "Ready"

**Example:**
```typescript
const state = kit.getState();
console.log("Current state:", state);
```

---

#### `getPosition(): number`
Returns the current position of the pipeline in nanoseconds.

**Returns:** Current position in nanoseconds

**Throws:** Error if pipeline is not initialized or query fails

**Example:**
```typescript
const positionNs = kit.getPosition();
const positionSeconds = positionNs / 1_000_000_000;
console.log(`Position: ${positionSeconds}s`);
```

---

#### `getDuration(): number`
Returns the duration of the pipeline in nanoseconds.

**Returns:** Duration in nanoseconds, or -1 if unknown

**Throws:** Error if pipeline is not initialized or query fails

**Example:**
```typescript
const durationNs = kit.getDuration();
const durationSeconds = durationNs / 1_000_000_000;
console.log(`Duration: ${durationSeconds}s`);
```

---

#### `seek(positionNs: number): void`
Seeks to a specific position in the pipeline.

**Parameters:**
- `positionNs`: Position to seek to in nanoseconds

**Throws:** Error if pipeline is not initialized or seek fails

**Example:**
```typescript
// Seek to 5 seconds (5 * 1_000_000_000 nanoseconds)
kit.seek(5_000_000_000);

// Seek to 10 seconds
kit.seek(10 * 1_000_000_000);
```

---

#### `setProperty(elementName: string, propertyName: string, value: string): void`
Sets a property on a named element in the pipeline.

**Parameters:**
- `elementName`: The name of the element
- `propertyName`: The name of the property to set
- `value`: The value to set (as a string)

**Throws:** Error if pipeline is not initialized, element not found, or property set fails

**Example:**
```typescript
kit.setProperty("mysrc", "is-live", "true");
kit.setProperty("mysrc", "format", "time");
```

---

#### `getProperty(elementName: string, propertyName: string): string`
Gets a property value from a named element in the pipeline.

**Parameters:**
- `elementName`: The name of the element
- `propertyName`: The name of the property to get

**Returns:** The property value as a string

**Throws:** Error if pipeline is not initialized, element not found, or property get fails

**Example:**
```typescript
const isLive = kit.getProperty("mysrc", "is-live");
console.log("is-live:", isLive);
```

---

#### `getElements(): string[]`
Returns a list of all element names in the pipeline.

**Returns:** Array of element names

**Throws:** Error if pipeline is not initialized

**Example:**
```typescript
const elements = kit.getElements();
console.log("Pipeline elements:", elements);
// Output: ["mysrc", "videoconvert", "mysink"]
```

---

#### `isInitialized(): boolean`
Checks if the pipeline has been initialized.

**Returns:** `true` if pipeline is initialized, `false` otherwise

**Example:**
```typescript
if (kit.isInitialized()) {
  console.log("Pipeline is ready");
}
```

---

#### `cleanup(): void`
Cleans up and releases the pipeline. Stops the pipeline and releases all resources.

**Throws:** Error if state change fails during cleanup

**Example:**
```typescript
kit.cleanup();
// After cleanup, you must call setPipeline again
kit.setPipeline("videotestsrc ! appsink name=sink");
```

---

## Usage Examples

### Basic Playback

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

// Create a simple pipeline: Video Test Source -> RGBA conversion -> AppSink
kit.setPipeline("videotestsrc ! video/x-raw,format=RGBA ! appsink name=mysink");

kit.play();

// Pull frames at ~30 FPS
setInterval(() => {
  const frame = kit.pullSample("mysink");
  if (frame) {
    console.log("Got frame of size:", frame.length);
  }
}, 33);
```

### Media File Playback with Seeking

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

// Play a video file
kit.setPipeline("filesrc location=video.mp4 ! decodebin ! videoconvert ! appsink name=sink");

kit.play();

// Get duration
const durationNs = kit.getDuration();
console.log(`Duration: ${durationNs / 1_000_000_000}s`);

// Seek to 10 seconds
setTimeout(() => {
  kit.seek(10_000_000_000);
  console.log("Seeked to 10s");
}, 5000);

// Monitor position
setInterval(() => {
  const positionNs = kit.getPosition();
  console.log(`Position: ${positionNs / 1_000_000_000}s`);
}, 1000);
```

### Custom Source with AppSrc

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

// Create a pipeline with AppSrc
kit.setPipeline("appsrc name=mysrc ! videoconvert ! autovideosink");

// Set AppSrc properties
kit.setProperty("mysrc", "is-live", "true");
kit.setProperty("mysrc", "format", "time");

kit.play();

// Push custom data to the pipeline
setInterval(() => {
  const data = generateFrameData(); // Your custom data generation
  kit.pushSample("mysrc", Buffer.from(data));
}, 33);
```

### Pipeline Inspection

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

kit.setPipeline("videotestsrc ! videoconvert ! autovideosink");

// List all elements in the pipeline
const elements = kit.getElements();
console.log("Pipeline elements:", elements);

// Get and set properties
for (const element of elements) {
  try {
    const isLive = kit.getProperty(element, "is-live");
    console.log(`${element}.is-live:`, isLive);
  } catch (e) {
    // Property may not exist
  }
}

kit.play();
```

### State Management

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

kit.setPipeline("videotestsrc ! autovideosink");

// Check initialization
console.log("Initialized:", kit.isInitialized()); // true

// Play
kit.play();
console.log("State:", kit.getState()); // "Playing"

// Pause
kit.pause();
console.log("State:", kit.getState()); // "Paused"

// Resume
kit.play();
console.log("State:", kit.getState()); // "Playing"

// Stop
kit.stop();
console.log("State:", kit.getState()); // "Null"

// Cleanup
kit.cleanup();
console.log("Initialized:", kit.isInitialized()); // false
```

### Error Handling

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

try {
  kit.setPipeline("invalid ! pipeline");
} catch (error) {
  console.error("Failed to set pipeline:", error.message);
}

try {
  kit.play(); // Will fail if pipeline not set
} catch (error) {
  console.error("Failed to play:", error.message);
}

try {
  kit.pullSample("nonexistent");
} catch (error) {
  console.error("Failed to pull sample:", error.message);
}
```

## Notes

- Always call `new GstKit()` before using any other methods
- Call `setPipeline()` before playback control methods (`play`, `pause`, `stop`)
- The pipeline is automatically cleaned up when the `GstKit` instance is garbage collected
- Use `cleanup()` to explicitly release resources when done
- All time-related values (position, duration, seek) are in nanoseconds
- AppSink and AppSrc elements must be named using the `name=` parameter in the pipeline string
