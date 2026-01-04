# GStreamer Kit (Native Module)

The `GstKit` class provides a generic, agnostic wrapper around GStreamer pipelines for Node.js applications.

## API Documentation

### Class: `GstKit`

#### `constructor()`
Initializes the GStreamer library. Must be called before using other methods.

#### `setPipeline(pipelineString: string): void`
Constructs a GStreamer pipeline from a launch string.
*   **Args**:
    *   `pipelineString`: Valid GStreamer pipeline description (e.g., `videotestsrc ! appsink name=sink`).

#### `play(): void`
Sets the pipeline state to `PLAYING`.

#### `pause(): void`
Sets the pipeline state to `PAUSED`.

#### `stop(): void`
Sets the pipeline state to `NULL` (stopped).

#### `pullSample(elementName: string): Buffer | null`
Attempts to pull a sample buffer from a named `appsink` element in the pipeline.
*   **Args**:
    *   `elementName`: The name of the `appsink` element (e.g., "sink").
*   **Returns**: A `Buffer` containing the sample data, or `null` if no sample is available.

#### `getState(): string`
Returns the current state of the pipeline as a string (e.g., "Playing", "Paused", "Null").

## Usage Example

```typescript
import { GstKit } from './index.js';

const kit = new GstKit();

// Create a pipeline: Video Test Source -> RGBA conversion -> AppSink
kit.setPipeline("videotestsrc ! video/x-raw,format=RGBA ! appsink name=mysink");

kit.play();

setInterval(() => {
  const frame = kit.pullSample("mysink");
  if (frame) {
    console.log("Got frame of size:", frame.length);
  }
}, 33);
```
