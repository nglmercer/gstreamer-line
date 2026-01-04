import { describe, it, expect } from 'bun:test';
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
});
