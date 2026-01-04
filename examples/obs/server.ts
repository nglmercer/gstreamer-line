import { Logger } from "./logger.js";
import { GstKit } from "../../index.js";
import { isCodecDecoderAvailable, getCodecDecoder } from "../../__test__/setup.js";

// Simple logger instance - Enable debug mode to see detailed logs
const logger = new Logger(true, "[RTMP]");

// RTMP Constants
const RTMP_HANDSHAKE_SIZE = 1536;
const RTMP_VERSION = 3;

// Message Type IDs
const MSG_SET_CHUNK_SIZE = 1;
const MSG_ABORT = 2;
const MSG_ACK = 3;
const MSG_USER_CONTROL = 4;
const MSG_WINDOW_ACK_SIZE = 5;
const MSG_SET_PEER_BW = 6;
const MSG_AUDIO = 8;
const MSG_VIDEO = 9;
const MSG_AMF3_DATA = 15;
const MSG_AMF3_SHARED = 16;
const MSG_AMF3_CMD = 17;
const MSG_AMF0_DATA = 18;
const MSG_AMF0_SHARED = 19;
const MSG_AMF0_CMD = 20;
const MSG_AGGREGATE = 22;

enum HandshakeState {
  UNINITIALIZED,
  VERSION_SENT,
  ACK_SENT,
  HANDSHAKE_DONE,
}

// Configuration
const FRAME_RATE = 30; // Target frame rate for RTMP streams

// GStreamer pipeline for processing RTMP streams in memory
class StreamProcessor {
  private gstKit: GstKit | null = null;
  private pipelineInitialized: boolean = false;
  private streamingStarted: boolean = false;
  private streamingInterval: NodeJS.Timeout | null = null;
  private sequenceHeaderReceived: boolean = false;
  private frameCount: number = 0;
  private outputDir: string = "./temp_frames";

  constructor() {
    // Create output directory if it doesn't exist
    this.ensureOutputDirectory();
  }

  private async ensureOutputDirectory() {
    try {
      const dirPath = this.outputDir;
      const gitkeepPath = `${dirPath}/.gitkeep`;
      
      // Try to create the directory and .gitkeep file
      await Bun.write(gitkeepPath, "");
      
      // Verify the directory was created
      const exists = await Bun.file(gitkeepPath).exists();
      if (exists) {
        console.log(`✅ Output directory ready: ${dirPath}`);
      } else {
        console.warn(`⚠️  Could not create output directory: ${dirPath}`);
      }
    } catch (error) {
      console.error(`❌ Error creating output directory:`, error);
    }
  }


  async initialize(streamKey: string): Promise<void> {
    if (this.pipelineInitialized) {
      logger.warn(`Pipeline already initialized for stream: ${streamKey}`);
      return;
    }

    // Check if H.264 decoder is available
    const h264DecoderAvailable = await isCodecDecoderAvailable('h264');
    
    if (!h264DecoderAvailable) {
      logger.error("H.264 decoder is NOT available on this system!");
      logger.error("RTMP streaming requires H.264 decoder support.");
      logger.error("Please install GStreamer plugins: gstreamer1.0-libav or gstreamer1.0-plugins-bad");
      logger.error("On Ubuntu/Debian: sudo apt-get install gstreamer1.0-libav gstreamer1.0-plugins-bad");
      logger.error("On Fedora: sudo dnf install gstreamer1-libav gstreamer1-plugins-bad");
      throw new Error("H.264 decoder not available - cannot process RTMP stream");
    }

    try {
      this.gstKit = new GstKit();

      // Get the best available H.264 decoder
      const h264Decoder = await getCodecDecoder('h264');
      
      // Pipeline for RTMP H.264 data - using the detected decoder
      // RTMP sends H.264 in AVCC format (avc1), we need to convert to byte-stream
      // h264parse will handle the conversion from AVCC to byte-stream format
      // Output RGBA format for direct display in canvas
      const pipelineString = `
        appsrc name=src format=bytes is-live=true do-timestamp=true caps="video/x-h264,stream-format=avc,alignment=au" !
        h264parse !
        ${h264Decoder} !
        videoconvert !
        videoscale !
        video/x-raw,format=RGBA,width=320,height=240,framerate=30/1 !
        appsink name=sink emit-signals=true sync=false max-buffers=10 drop=true
      `;

      this.gstKit.setPipeline(pipelineString);
      
      // Start the pipeline immediately
      this.gstKit.play();
      
      // Wait a bit for pipeline to initialize
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Check state
      const state = this.gstKit?.getState();
      console.log(`Pipeline state: ${state}`);
      
      this.pipelineInitialized = true;
      logger.info(`GStreamer pipeline initialized for stream: ${streamKey} (using decoder: ${h264Decoder})`);
    } catch (error) {
      logger.error("Failed to initialize GStreamer pipeline", error);
      this.cleanup();
      throw error;
    }
  }

  // Start streaming when data is available
  startStreaming(): void {
    if (this.streamingStarted || !this.pipelineInitialized || !this.gstKit) {
      console.log('Cannot start streaming:', {
        streamingStarted: this.streamingStarted,
        pipelineInitialized: this.pipelineInitialized,
        hasGstKit: !!this.gstKit,
        sequenceHeaderReceived: this.sequenceHeaderReceived
      });
      return;
    }

    // Wait for sequence header before starting frame extraction
    if (!this.sequenceHeaderReceived) {
      console.log('Waiting for sequence header...');
      return;
    }

    this.streamingStarted = true;
    console.log('Starting frame extraction...');

    const frameInterval = 1000 / FRAME_RATE; // ms between frames (~33.33ms for 30fps)

    // Wait for decoder to initialize before starting the loop
    setTimeout(() => {
      this.streamingInterval = setInterval(() => {
      if (!this.pipelineInitialized || !this.gstKit) {
        return;
      }

      try {
        const frame = this.gstKit.pullSample("sink", 100); // 100ms timeout

        if (frame) {
          this.frameCount++;
          
          console.log(`✅ Pulled frame ${this.frameCount}, size: ${frame.length} bytes`);
          
          // Save frame as temporary image file
          this.saveFrameAsImage(frame);

          // Log progress every 30 frames
          if (this.frameCount % 30 === 0) {
            console.log(`📹 Processed ${this.frameCount} frames`);
          }
        } else {
          // Log occasionally that we're waiting for frames
          if (this.frameCount % 100 === 0) {
            console.log('⏳ Waiting for frames from pipeline...');
          }
        }
      } catch (error) {
        console.error("Error pulling frame:", error);
      }
    }, frameInterval);
    }, 1000); // 1 second delay to allow decoder to initialize
  }

  private async saveFrameAsImage(frameBuffer: Buffer) {
    try {
      // Create filename with timestamp and frame number
      const timestamp = Date.now();
      const filename = `${this.outputDir}/frame_${this.frameCount.toString().padStart(6, '0')}_${timestamp}.rgba`;
      
      // Write the RGBA buffer to a file
      await Bun.write(filename, frameBuffer);
      
      // Verify the file was written
      const fileExists = await Bun.file(filename).exists();
      
      // Log every 30 frames
      if (this.frameCount % 30 === 0) {
        console.log(`💾 Saved frame ${this.frameCount} to ${filename} (${frameBuffer.length} bytes, exists: ${fileExists})`);
      }
    } catch (error) {
      console.error('❌ Error saving frame:', error);
    }
  }

  processVideoBuffer(buffer: Buffer): void {
    if (!this.pipelineInitialized || !this.gstKit) {
      console.log('⚠️ Pipeline not initialized, cannot process video buffer');
      return;
    }

    try {
      // RTMP video data structure:
      // Byte 0: Frame type (keyframe=0x17, interframe=0x27)
      // Byte 1: AVC packet type (0=sequence header, 1=NALU, 2=end of sequence)
      // Bytes 2-3: Composition time (24-bit)
      // Bytes 4+: H.264 data in AVCC format (with length prefixes)
      
      if (buffer.length > 4) {
        const avcPacketType = buffer[1];
        
        // Skip RTMP header (4 bytes) and extract H.264 data
        const h264Data = buffer.subarray(4);
        
        console.log(`📥 Processing video buffer: ${buffer.length} bytes, AVC type: ${avcPacketType}, H.264 data: ${h264Data.length} bytes`);
        
        // Check if this is a sequence header (SPS/PPS)
        if (avcPacketType === 0 && !this.sequenceHeaderReceived) {
          this.sequenceHeaderReceived = true;
          console.log('✅ Sequence header received');
        }
        
        this.gstKit.pushSample("src", h264Data);
        console.log(`✅ Pushed ${h264Data.length} bytes to GStreamer pipeline`);
        
        // Start streaming after sequence header is received
        if (this.sequenceHeaderReceived && !this.streamingStarted) {
          this.startStreaming();
        }
      }
    } catch (error) {
      console.error('❌ Error processing video buffer:', error);
    }
  }

  processAudioBuffer(buffer: Buffer): void {
    // Audio is not processed, only video
    return;
  }

  // Method to check pipeline status
  getPipelineStatus(): { initialized: boolean; frameCount: number } {
    return {
      initialized: this.pipelineInitialized,
      frameCount: this.frameCount
    };
  }

  cleanup(): void {
    if (this.streamingInterval) {
      clearInterval(this.streamingInterval);
      this.streamingInterval = null;
    }
    if (this.pipelineInitialized && this.gstKit) {
      this.gstKit.stop();
      this.gstKit.cleanup();
      this.gstKit = null;
      this.pipelineInitialized = false;
      logger.info("GStreamer pipeline cleaned up");
    }
  }
}

// Global stream processor
const streamProcessor = new StreamProcessor();

class RTMPConnection {
  private socket: any;
  private buffer: Buffer = Buffer.alloc(0);
  private handshakeState: HandshakeState = HandshakeState.UNINITIALIZED;
  private clientId: string;
  private chunkSize: number = 128;
  private peerChunkSize: number = 128;
  private windowAckSize: number = 2500000;
  private peerBandwidth: number = 2500000;
  private bytesReceived: number = 0;
  private lastAckSent: number = 0;
  private streamKey: string | null = null;

  private incompleteMessages: Map<
    number,
    {
      buffer: Buffer;
      bytesReceived: number;
      totalLength: number;
      messageType: number;
      timestamp: number;
      streamId: number;
    }
  > = new Map();

  private lastMessageLength: Map<number, number> = new Map();
  private lastMessageType: Map<number, number> = new Map();
  private lastMessageStreamId: Map<number, number> = new Map();
  private lastTimestamp: Map<number, number> = new Map();

  constructor(socket: any, clientId: string) {
    this.socket = socket;
    this.clientId = clientId;
  }

  async handleData(data: Buffer | Uint8Array) {
    const bufferData = Buffer.isBuffer(data) ? data : Buffer.from(data);

    this.buffer = Buffer.concat([this.buffer, bufferData]);
    this.bytesReceived += bufferData.length;

    if (this.handshakeState !== HandshakeState.HANDSHAKE_DONE) {
      this.processHandshake();
    } else {
      await this.processRTMPMessages();
    }
  }

  private async processHandshake() {
    logger.log(`Handshake state: ${HandshakeState[this.handshakeState]}`);

    switch (this.handshakeState) {
      case HandshakeState.UNINITIALIZED:
        const needed = 1 + RTMP_HANDSHAKE_SIZE;

        if (this.buffer.length >= needed) {
          const version = this.buffer[0];

          if (version !== RTMP_VERSION) {
            logger.error(`Invalid RTMP version: ${version}`);
            this.socket.end();
            return;
          }

          const c1 = this.buffer.subarray(1, 1 + RTMP_HANDSHAKE_SIZE);
          this.buffer = this.buffer.subarray(1 + RTMP_HANDSHAKE_SIZE);

          // S0 + S1 + S2
          const s0 = Buffer.from([RTMP_VERSION]);

          const s1 = Buffer.alloc(RTMP_HANDSHAKE_SIZE);
          s1.writeUInt32BE(Math.floor(Date.now() / 1000), 0);
          s1.writeUInt32BE(0, 4);
          for (let i = 8; i < RTMP_HANDSHAKE_SIZE; i++) {
            s1[i] = Math.floor(Math.random() * 256);
          }

          const s2 = Buffer.from(c1);

          const response = Buffer.concat([s0, s1, s2]);
          this.socket.write(response);

          this.handshakeState = HandshakeState.ACK_SENT;
        }
        break;

      case HandshakeState.ACK_SENT:
        if (this.buffer.length >= RTMP_HANDSHAKE_SIZE) {
          const c2 = this.buffer.subarray(0, RTMP_HANDSHAKE_SIZE);
          this.buffer = this.buffer.subarray(RTMP_HANDSHAKE_SIZE);

          this.handshakeState = HandshakeState.HANDSHAKE_DONE;
          logger.info("Handshake completed");

          this.sendServerConfig();

          if (this.buffer.length > 0) {
            this.processRTMPMessages();
          }
        }
        break;
    }
  }

  private async sendServerConfig() {
    logger.log("Sending server configuration");

    this.sendSetChunkSize(4096);
    this.sendWindowAckSize(this.windowAckSize);
    this.sendSetPeerBandwidth(this.peerBandwidth, 2);
  }

  private sendSetChunkSize(size: number) {
    const payload = Buffer.alloc(4);
    payload.writeUInt32BE(size, 0);
    this.sendControlMessage(2, MSG_SET_CHUNK_SIZE, payload);
    this.chunkSize = size;
  }

  private sendWindowAckSize(size: number) {
    const payload = Buffer.alloc(4);
    payload.writeUInt32BE(size, 0);
    this.sendControlMessage(2, MSG_WINDOW_ACK_SIZE, payload);
  }

  private sendSetPeerBandwidth(size: number, limitType: number) {
    const payload = Buffer.alloc(5);
    payload.writeUInt32BE(size, 0);
    payload.writeUInt8(limitType, 4);
    this.sendControlMessage(2, MSG_SET_PEER_BW, payload);
  }

  private sendControlMessage(
    csid: number,
    messageType: number,
    payload: Buffer,
  ) {
    const header = Buffer.alloc(12);
    header[0] = (0 << 6) | (csid & 0x3f);
    header.writeUIntBE(0, 1, 3);
    header.writeUIntBE(payload.length, 4, 3);
    header[7] = messageType;
    header.writeUInt32LE(0, 8);

    const message = Buffer.concat([header, payload]);
    this.socket.write(message);
  }

  private async processRTMPMessages() {
    await this.processRTMPMessagesLoop();
  }

  private async processRTMPMessagesLoop() {
    let loopCount = 0;
    while (this.buffer.length > 0) {
      const startLen = this.buffer.length;
      loopCount++;

      if (this.buffer.length < 1) {
        logger.log(`Loop ${loopCount}: Buffer too short (${this.buffer.length} bytes)`);
        break;
      }

      const basicHeader = this.buffer[0];
      if (!basicHeader) break;

      const fmt = (basicHeader >> 6) & 0x03;
      let csid = basicHeader & 0x3f;
      let offset = 1;

      if (csid === 0) {
        if (this.buffer.length < 2) break;
        const nextByte = this.buffer[1];
        if (nextByte === undefined) break;
        csid = nextByte + 64;
        offset = 2;
      } else if (csid === 1) {
        if (this.buffer.length < 3) break;
        const byte1 = this.buffer[1];
        const byte2 = this.buffer[2];
        if (byte1 === undefined || byte2 === undefined) break;
        csid = (byte2 << 8) + byte1 + 64;
        offset = 3;
      }

      let headerSize = offset;
      let timestamp = this.lastTimestamp.get(csid) || 0;
      let messageLength = this.lastMessageLength.get(csid) || 0;
      let messageType = this.lastMessageType.get(csid) || 0;
      let streamId = this.lastMessageStreamId.get(csid) || 0;

      if (fmt === 0) {
        headerSize += 11;
        if (this.buffer.length < headerSize) {
          logger.log(`Loop ${loopCount}: Need ${headerSize} bytes for type 0 header, have ${this.buffer.length}`);
          break;
        }

        timestamp = this.buffer.readUIntBE(offset, 3);
        messageLength = this.buffer.readUIntBE(offset + 3, 3);
        const msgType = this.buffer[offset + 6];
        if (msgType !== undefined) {
          messageType = msgType;
        }
        streamId = this.buffer.readUInt32LE(offset + 7);

        this.lastTimestamp.set(csid, timestamp);
        this.lastMessageLength.set(csid, messageLength);
        this.lastMessageType.set(csid, messageType);
        this.lastMessageStreamId.set(csid, streamId);
      } else if (fmt === 1) {
        headerSize += 7;
        if (this.buffer.length < headerSize) {
          logger.log(`Loop ${loopCount}: Need ${headerSize} bytes for type 1 header, have ${this.buffer.length}`);
          break;
        }

        const timestampDelta = this.buffer.readUIntBE(offset, 3);
        timestamp += timestampDelta;
        messageLength = this.buffer.readUIntBE(offset + 3, 3);
        const msgType = this.buffer[offset + 6];
        if (msgType !== undefined) {
          messageType = msgType;
        }

        this.lastTimestamp.set(csid, timestamp);
        this.lastMessageLength.set(csid, messageLength);
        this.lastMessageType.set(csid, messageType);
      } else if (fmt === 2) {
        headerSize += 3;
        if (this.buffer.length < headerSize) {
          logger.log(`Loop ${loopCount}: Need ${headerSize} bytes for type 2 header, have ${this.buffer.length}`);
          break;
        }

        const timestampDelta = this.buffer.readUIntBE(offset, 3);
        timestamp += timestampDelta;
        this.lastTimestamp.set(csid, timestamp);
      }

      const incomplete = this.incompleteMessages.get(csid);
      const remainingBytes = incomplete
        ? incomplete.totalLength - incomplete.bytesReceived
        : messageLength;

      const bytesToRead = Math.min(remainingBytes, this.peerChunkSize);

      if (this.buffer.length < headerSize + bytesToRead) {
        logger.log(`Loop ${loopCount}: Need ${headerSize + bytesToRead} bytes for data, have ${this.buffer.length}`);
        break;
      }

      const chunkData = Buffer.from(
        this.buffer.subarray(headerSize, headerSize + bytesToRead),
      );
      this.buffer = this.buffer.subarray(headerSize + bytesToRead);

      if (!incomplete) {
        if (messageLength <= bytesToRead) {
          await this.handleCompleteMessage(messageType, chunkData, csid, streamId);
        } else {
          this.incompleteMessages.set(csid, {
            buffer: chunkData,
            bytesReceived: bytesToRead,
            totalLength: messageLength,
            messageType,
            timestamp,
            streamId,
          });
          logger.log(`Loop ${loopCount}: Started assembling message type ${messageType}, received ${bytesToRead}/${messageLength} bytes`);
        }
      } else {
        incomplete.buffer = Buffer.concat([incomplete.buffer, chunkData]);
        incomplete.bytesReceived += bytesToRead;

        if (incomplete.bytesReceived >= incomplete.totalLength) {
          await this.handleCompleteMessage(
            incomplete.messageType,
            incomplete.buffer,
            csid,
            incomplete.streamId,
          );
          this.incompleteMessages.delete(csid);
        } else {
          logger.log(`Loop ${loopCount}: Continuing assembly of message type ${incomplete.messageType}, received ${incomplete.bytesReceived}/${incomplete.totalLength} bytes`);
        }
      }

      if (this.bytesReceived - this.lastAckSent >= this.windowAckSize) {
        this.sendAck(this.bytesReceived);
        this.lastAckSent = this.bytesReceived;
      }

      if (this.buffer.length === startLen) {
        logger.log(`Loop ${loopCount}: No progress made, buffer length unchanged at ${this.buffer.length}`);
        break;
      }
    }
  }

  private async handleCompleteMessage(
    messageType: number,
    payload: Buffer,
    csid: number,
    streamId: number,
  ) {
    logger.log(`Received message type ${messageType} (${this.getMessageTypeName(messageType)}) with ${payload.length} bytes`);

    switch (messageType) {
      case MSG_SET_CHUNK_SIZE:
        if (payload.length >= 4) {
          this.peerChunkSize = payload.readUInt32BE(0) & 0x7fffffff;
          logger.log(`Chunk size set to: ${this.peerChunkSize}`);
        }
        break;

      case MSG_WINDOW_ACK_SIZE:
        if (payload.length >= 4) {
          const size = payload.readUInt32BE(0);
          logger.log(`Window ACK size: ${size}`);
        }
        break;

      case MSG_SET_PEER_BW:
        if (payload.length >= 5) {
          const size = payload.readUInt32BE(0);
          const limit = payload[4];
          logger.log(`Peer bandwidth: ${size}, limit: ${limit}`);
        }
        break;

      case MSG_AMF0_DATA:
      case MSG_AMF3_DATA:
        logger.log(`Received metadata/data message (AMF0/AMF3)`);
        // Send acknowledgment for metadata to tell OBS we received it
        // This might trigger OBS to start sending video data
        this.sendAck(this.bytesReceived);
        logger.log(`Sent ACK for metadata (${this.bytesReceived} bytes)`);
        break;

      case MSG_AMF0_CMD:
      case MSG_AMF3_CMD:
        await this.handleCommand(
          payload,
          csid,
          streamId,
          messageType === MSG_AMF3_CMD,
        );
        break;

      case MSG_AUDIO:
        logger.log(`[RTMP AUDIO] Received ${payload.length} bytes from OBS`);
        streamProcessor.processAudioBuffer(payload);
        break;

      case MSG_VIDEO:
        logger.log(`[RTMP VIDEO] Received ${payload.length} bytes from OBS`);
        streamProcessor.processVideoBuffer(payload);
        break;

      default:
        logger.log(`Unhandled message type: ${messageType} (${this.getMessageTypeName(messageType)})`);
    }
  }

  private getMessageTypeName(messageType: number): string {
    const names: Record<number, string> = {
      1: 'SET_CHUNK_SIZE',
      2: 'ABORT',
      3: 'ACK',
      4: 'USER_CONTROL',
      5: 'WINDOW_ACK_SIZE',
      6: 'SET_PEER_BW',
      8: 'AUDIO',
      9: 'VIDEO',
      15: 'AMF3_DATA',
      16: 'AMF3_SHARED',
      17: 'AMF3_CMD',
      18: 'AMF0_DATA',
      19: 'AMF0_SHARED',
      20: 'AMF0_CMD',
      22: 'AGGREGATE',
    };
    return names[messageType] || `UNKNOWN(${messageType})`;
  }

  private sendAck(bytes: number) {
    const payload = Buffer.alloc(4);
    payload.writeUInt32BE(bytes, 0);
    this.sendControlMessage(2, MSG_ACK, payload);
  }

  private sendStreamBegin() {
    const payload = Buffer.alloc(6);
    payload.writeUInt16BE(0, 0);
    payload.writeUInt32BE(0, 2);
    this.sendControlMessage(4, MSG_USER_CONTROL, payload);
    logger.log("Sent StreamBegin user control message");
  }

  private async handleCommand(
    payload: Buffer,
    csid: number,
    streamId: number,
    isAMF3: boolean,
  ) {
    try {
      let offset = 0;
      if (isAMF3 && payload[0] === 0) {
        offset = 1;
      }

      const { command, transactionId, args } = this.parseAMF0(
        payload.subarray(offset),
      );

      switch (command) {
        case "connect":
          this.handleConnect(csid, transactionId, args);
          break;

        case "releaseStream":
          this.sendCommandResponse(csid, "_result", transactionId, null, null);
          break;

        case "FCPublish":
          this.sendCommandResponse(csid, "_result", transactionId, null, null);
          break;

        case "createStream":
          const streamId = 1;
          this.sendCommandResponse(csid, "_result", transactionId, null, streamId);
          // Send StreamBegin with actual stream ID
          const streamBegin = Buffer.alloc(6);
          streamBegin.writeUInt16BE(0, 0);
          streamBegin.writeUInt32BE(streamId, 2);
          this.sendControlMessage(4, MSG_USER_CONTROL, streamBegin);
          logger.log(`Sent StreamBegin for stream ID ${streamId}`);
          break;

        case "publish":
          this.handlePublish(csid, args);
          break;

        default:
          logger.log(`Unknown command: ${command}`);
      }
    } catch (error: any) {
      logger.error(`Error parsing command: ${error.message}`);
    }
  }

  private parseAMF0(buffer: Buffer): {
    command: string;
    transactionId: number;
    args: any[];
  } {
    let offset = 0;
    const args: any[] = [];

    if (buffer[offset] !== 0x02) {
      throw new Error(
        `Expected string marker (0x02), got 0x${(buffer[offset] || 0).toString(16)}`,
      );
    }
    const cmdLen = buffer.readUInt16BE(offset + 1);
    const command = buffer.toString("utf8", offset + 3, offset + 3 + cmdLen);
    offset += 3 + cmdLen;

    if (buffer[offset] !== 0x00) {
      throw new Error(
        `Expected number marker (0x00), got 0x${(buffer[offset] || 0).toString(16)}`,
      );
    }
    const transactionId = buffer.readDoubleBE(offset + 1);
    offset += 9;

    while (offset < buffer.length - 1) {
      const type = buffer[offset];

      if (type === 0x02) {
        const len = buffer.readUInt16BE(offset + 1);
        const value = buffer.toString("utf8", offset + 3, offset + 3 + len);
        args.push(value);
        offset += 3 + len;
      } else if (type === 0x05) {
        args.push(null);
        offset += 1;
      } else if (type === 0x03) {
        const obj: any = {};
        offset += 1;

        while (offset < buffer.length - 2) {
          if (
            buffer[offset] === 0x00 &&
            buffer[offset + 1] === 0x00 &&
            buffer[offset + 2] === 0x09
          ) {
            offset += 3;
            break;
          }

          const propLen = buffer.readUInt16BE(offset);
          const propName = buffer.toString(
            "utf8",
            offset + 2,
            offset + 2 + propLen,
          );
          offset += 2 + propLen;

          const propType = buffer[offset];
          if (propType === 0x02) {
            const valLen = buffer.readUInt16BE(offset + 1);
            obj[propName] = buffer.toString(
              "utf8",
              offset + 3,
              offset + 3 + valLen,
            );
            offset += 3 + valLen;
          } else if (propType === 0x00) {
            obj[propName] = buffer.readDoubleBE(offset + 1);
            offset += 9;
          } else if (propType === 0x01) {
            obj[propName] = buffer[offset + 1] !== 0;
            offset += 2;
          } else {
            offset++;
          }
        }
        args.push(obj);
      } else if (type === 0x00) {
        const value = buffer.readDoubleBE(offset + 1);
        args.push(value);
        offset += 9;
      } else {
        break;
      }
    }

    return { command, transactionId, args };
  }

  private async handleConnect(
    csid: number,
    transactionId: number,
    args: any[],
  ) {
    // Don't send StreamBegin here - it should be sent after createStream
    // with the actual stream ID

    this.sendCommandResponse(
      csid,
      "_result",
      transactionId,
      {
        fmsVer: "FMS/3,5,7,7009",
        capabilities: 31,
        mode: 1,
      },
      {
        level: "status",
        code: "NetConnection.Connect.Success",
        description: "Connection succeeded",
        objectEncoding: 0,
      },
    );

    logger.info("Connection successful");
  }

  private async handlePublish(csid: number, args: any[]) {
    const streamKey = args[0] || "default";
    this.streamKey = streamKey;

    logger.info(`Stream published: ${streamKey}`);
    logger.info(`Waiting for video data from RTMP client...`);

    // Send publish response immediately, don't block on initialization
    this.sendCommandResponse(csid, "onStatus", 0, null, {
      level: "status",
      code: "NetStream.Publish.Start",
      description: "Stream is now published",
      details: streamKey,
    });

    // Send StreamBegin message to tell OBS we're ready to receive video data
    // This is sent after publish to signal that the stream is ready
    const streamBeginPayload = Buffer.alloc(6);
    streamBeginPayload.writeUInt16BE(0, 0);
    streamBeginPayload.writeUInt32BE(1, 0);
    this.sendControlMessage(4, MSG_USER_CONTROL, streamBeginPayload);
    logger.log("Sent StreamBegin for stream ID 1 after publish");

    // Initialize GStreamer pipeline in the background without blocking
    streamProcessor.initialize(streamKey).catch(error => {
      logger.error(`Failed to initialize GStreamer pipeline: ${error}`);
    });
  }

  private sendCommandResponse(
    csid: number,
    command: string,
    transactionId: number,
    ...args: any[]
  ) {
    const payload = this.encodeAMF0(command, transactionId, ...args);

    const header = Buffer.alloc(12);
    header[0] = (0 << 6) | (csid & 0x3f);
    header.writeUIntBE(0, 1, 3);
    header.writeUIntBE(payload.length, 4, 3);
    header[7] = MSG_AMF0_CMD;
    header.writeUInt32LE(0, 8);

    this.socket.write(Buffer.concat([header, payload]));
  }

  private encodeAMF0(
    command: string,
    transactionId: number,
    ...args: any[]
  ): Buffer {
    const buffers: Buffer[] = [];

    buffers.push(Buffer.from([0x02]));
    const cmdBuf = Buffer.from(command, "utf8");
    const cmdLen = Buffer.allocUnsafe(2);
    cmdLen.writeUInt16BE(cmdBuf.length);
    buffers.push(cmdLen, cmdBuf);

    buffers.push(Buffer.from([0x00]));
    const tidBuf = Buffer.allocUnsafe(8);
    tidBuf.writeDoubleBE(transactionId);
    buffers.push(tidBuf);

    for (const arg of args) {
      if (arg === null || arg === undefined) {
        buffers.push(Buffer.from([0x05]));
      } else if (typeof arg === "number") {
        buffers.push(Buffer.from([0x00]));
        const numBuf = Buffer.allocUnsafe(8);
        numBuf.writeDoubleBE(arg);
        buffers.push(numBuf);
      } else if (typeof arg === "boolean") {
        buffers.push(Buffer.from([0x01]));
        buffers.push(Buffer.from([arg ? 1 : 0]));
      } else if (typeof arg === "object") {
        buffers.push(Buffer.from([0x03]));

        for (const [key, value] of Object.entries(arg)) {
          const keyBuf = Buffer.from(key, "utf8");
          const keyLen = Buffer.allocUnsafe(2);
          keyLen.writeUInt16BE(keyBuf.length);
          buffers.push(keyLen, keyBuf);

          if (typeof value === "string") {
            buffers.push(Buffer.from([0x02]));
            const valBuf = Buffer.from(value, "utf8");
            const valLen = Buffer.allocUnsafe(2);
            valLen.writeUInt16BE(valBuf.length);
            buffers.push(valLen, valBuf);
          } else if (typeof value === "number") {
            buffers.push(Buffer.from([0x00]));
            const valBuf = Buffer.allocUnsafe(8);
            valBuf.writeDoubleBE(value);
            buffers.push(valBuf);
          } else if (typeof value === "boolean") {
            buffers.push(Buffer.from([0x01]));
            buffers.push(Buffer.from([value ? 1 : 0]));
          }
        }

        buffers.push(Buffer.from([0x00, 0x00, 0x09]));
      }
    }

    return Buffer.concat(buffers);
  }
}

class RTMPServer {
  private port: number;

  constructor(port: number = 1935) {
    this.port = port;
    this.startTCPServer();
  }

  private startTCPServer() {
    try {
      Bun.listen({
        hostname: "0.0.0.0",
        port: this.port,
        socket: {
          open: (socket: any) => {
            const clientId = `${socket.remoteAddress}:${socket.remotePort}`;
            const conn = new RTMPConnection(socket, clientId);
            socket.data = conn;
            logger.info(`Client connected: ${clientId}`);
          },

          data: (socket: any, receivedData: Buffer) => {
            const conn = socket.data as RTMPConnection | undefined;

            if (!conn) {
              logger.error(`Connection not found for socket`);
              return;
            }

            logger.log(`Socket data event: ${receivedData.length} bytes received`);
            conn.handleData(receivedData).catch(error => {
              logger.error(`Error handling data: ${error}`);
            });
            logger.log(`After handleData: buffer size = ${conn['buffer']?.length || 0}`);
          },

          close: (socket: any) => {
            const clientId = `${socket.remoteAddress}:${socket.remotePort}`;
            logger.info(`Client disconnected: ${clientId}`);
          },

          error: (socket: any, error: any) => {
            logger.error(`Socket error: ${error}`);
          },
        },
      });

      logger.info(`RTMP Server started on rtmp://localhost:${this.port}`);
      logger.info(`OBS Settings:`);
      logger.info(`  Server: rtmp://localhost:${this.port}/live`);
      logger.info(`  Stream Key: any_key`);
    } catch (error: any) {
      logger.error(`Fatal error: ${error.message}`);
      process.exit(1);
    }
  }

}

export { RTMPServer, RTMPConnection, StreamProcessor };

// Function to get the stream processor instance
export function getStreamProcessor(): StreamProcessor {
  return streamProcessor;
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\n\n👋 Shutting down...');
  streamProcessor.cleanup();
  process.exit(0);
});
