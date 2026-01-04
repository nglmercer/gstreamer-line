import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { GstKit } from '../index.js';

describe('GstKit', () => {
  it('should initialize without error', () => {
    const kit = new GstKit();
    expect(kit).toBeDefined();
    expect(kit.getState()).toBe('Null');
  });

  it('should handle simple pipeline', () => {
    const kit = new GstKit();
    // Simple pipeline: fakesrc returning one buffer then EOS
    kit.setPipeline('fakesrc num-buffers=1 ! appsink name=test_sink');

    expect(kit.getState()).toBe('Null');

    kit.play();
    expect(kit.getState()).not.toBe('Null'); // Playing or Paused (preroll)

    kit.stop();
    expect(kit.getState()).toBe('Null');
  });

  it('should throw on invalid pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.setPipeline('this is not a valid pipeline');
    }).toThrow();
  });

  it('should check if initialized', () => {
    const kit = new GstKit();
    expect(kit.isInitialized()).toBe(false);

    kit.setPipeline('fakesrc num-buffers=1 ! appsink name=sink');
    expect(kit.isInitialized()).toBe(true);

    kit.cleanup();
    expect(kit.isInitialized()).toBe(false);
  });

  it('should pause and resume pipeline', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=10 ! appsink name=sink');

    kit.play();
    expect(kit.getState()).toBe('Playing');

    kit.pause();
    expect(kit.getState()).toBe('Paused');

    kit.play();
    expect(kit.getState()).toBe('Playing');

    kit.stop();
  });

  it('should pull samples from appsink', async () => {
    const kit = new GstKit();
    kit.setPipeline('videotestsrc num-buffers=5 ! video/x-raw,format=RGBA ! appsink name=sink');

    kit.play();

    // Wait for pipeline to process
    await new Promise(resolve => setTimeout(resolve, 200));

    let samplesReceived = 0;
    for (let i = 0; i < 10; i++) {
      const sample = kit.pullSample('sink');
      if (sample) {
        samplesReceived++;
        expect(sample.length).toBeGreaterThan(0);
      }
    }

    expect(samplesReceived).toBeGreaterThan(0);
    kit.stop();
  });

  it('should return null when no sample available', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=0 ! appsink name=sink');

    kit.play();
    const sample = kit.pullSample('sink');
    expect(sample).toBeNull();

    kit.stop();
  });

  it('should throw when pulling from non-existent element', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 ! appsink name=sink');

    expect(() => {
      kit.pullSample('nonexistent');
    }).toThrow();

    kit.stop();
  });

  it('should throw when pulling from non-appsink element', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 name=src ! appsink name=sink');

    expect(() => {
      kit.pullSample('src');
    }).toThrow();

    kit.stop();
  });

  it('should get position', async () => {
    const kit = new GstKit();
    kit.setPipeline('videotestsrc num-buffers=100 ! fakesink');

    kit.play();

    // Wait for pipeline to start
    await new Promise(resolve => setTimeout(resolve, 100));

    const position = kit.getPosition();
    expect(position).toBeGreaterThanOrEqual(0);

    kit.stop();
  });

  it('should get duration', async () => {
    const kit = new GstKit();
    kit.setPipeline('videotestsrc num-buffers=100 ! fakesink');

    kit.play();

    // Wait for pipeline to start
    await new Promise(resolve => setTimeout(resolve, 100));

    const duration = kit.getDuration();
    // Duration may be -1 if unknown, or a positive value
    expect(duration).toBeGreaterThanOrEqual(-1);

    kit.stop();
  });

  it('should get pipeline elements', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 ! fakesink');

    const elements = kit.getElements();
    expect(elements.length).toBeGreaterThan(0);
    expect(elements.some(e => e.startsWith('fakesrc'))).toBe(true);
    expect(elements.some(e => e.startsWith('fakesink'))).toBe(true);

    kit.stop();
  });

  it('should set and get properties', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 name=src ! fakesink');

    // Get existing property
    const numBuffers = kit.getProperty('src', 'num-buffers');
    expect(numBuffers).toBeDefined();

    // Set property
    kit.setProperty('src', 'num-buffers', '5');
    const updated = kit.getProperty('src', 'num-buffers');
    expect(updated).toContain('5');

    kit.stop();
  });

  it('should throw when setting property on non-existent element', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 ! fakesink');

    expect(() => {
      kit.setProperty('nonexistent', 'some-prop', 'value');
    }).toThrow();

    kit.stop();
  });

  it('should throw when getting property on non-existent element', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 ! fakesink');

    expect(() => {
      kit.getProperty('nonexistent', 'some-prop');
    }).toThrow();

    kit.stop();
  });

  it('should cleanup pipeline', () => {
    const kit = new GstKit();
    kit.setPipeline('fakesrc num-buffers=1 ! appsink name=sink');

    expect(kit.isInitialized()).toBe(true);

    kit.cleanup();
    expect(kit.isInitialized()).toBe(false);
    expect(kit.getState()).toBe('Null');
  });

  it('should handle multiple pipelines sequentially', () => {
    const kit = new GstKit();

    // First pipeline
    kit.setPipeline('fakesrc num-buffers=1 ! appsink name=sink1');
    kit.play();
    kit.stop();

    // Cleanup and create new pipeline
    kit.cleanup();
    kit.setPipeline('fakesrc num-buffers=2 ! appsink name=sink2');
    kit.play();
    kit.stop();
  });

  it('should throw when calling play without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.play();
    }).toThrow();
  });

  it('should throw when calling pause without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.pause();
    }).toThrow();
  });

  it('should throw when calling stop without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.stop();
    }).toThrow();
  });

  it('should throw when calling getPosition without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.getPosition();
    }).toThrow();
  });

  it('should throw when calling getDuration without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.getDuration();
    }).toThrow();
  });

  it('should throw when calling seek without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.seek(1000);
    }).toThrow();
  });

  it('should throw when calling getElements without pipeline', () => {
    const kit = new GstKit();
    expect(() => {
      kit.getElements();
    }).toThrow();
  });

  it('should handle videotestsrc pipeline', async () => {
    const kit = new GstKit();
    kit.setPipeline('videotestsrc num-buffers=10 ! video/x-raw,format=RGBA ! appsink name=sink');

    kit.play();

    // Wait for pipeline to process
    await new Promise(resolve => setTimeout(resolve, 100));

    const sample = kit.pullSample('sink');
    expect(sample).not.toBeNull();
    expect(sample!.length).toBeGreaterThan(0);

    kit.stop();
  });

  it('should handle pipeline with multiple elements', () => {
    const kit = new GstKit();
    kit.setPipeline('videotestsrc ! videoconvert ! fakesink');

    const elements = kit.getElements();
    expect(elements.length).toBeGreaterThan(0);

    kit.stop();
  });
});

describe('GstKit - AppSrc', () => {
  it('should push samples to appsrc', () => {
    const kit = new GstKit();
    kit.setPipeline('appsrc name=src ! fakesink');

    kit.setProperty('src', 'is-live', 'true');
    kit.setProperty('src', 'format', 'time');

    kit.play();

    // Push some data
    const data = Buffer.from([0, 1, 2, 3, 4, 5, 6, 7]);
    kit.pushSample('src', data);

    kit.stop();
  });

  it('should throw when pushing to non-existent element', () => {
    const kit = new GstKit();
    kit.setPipeline('appsrc name=src ! fakesink');

    expect(() => {
      kit.pushSample('nonexistent', Buffer.from([0, 1, 2]));
    }).toThrow();

    kit.stop();
  });

  it('should throw when pushing to non-appsrc element', () => {
    const kit = new GstKit();
    kit.setPipeline('appsrc name=src ! fakesink name=sink');

    expect(() => {
      kit.pushSample('sink', Buffer.from([0, 1, 2]));
    }).toThrow();

    kit.stop();
  });
});
