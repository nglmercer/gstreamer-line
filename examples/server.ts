import { Logger } from "./logger.js";
import { GstKit } from "../index.js";

// Simple logger instance
const logger = new Logger(false, "[RTMP]");

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

// GStreamer pipeline for processing RTMP streams in memory
class StreamProcessor {
  private gstKit: GstKit;
  private pipelineInitialized: boolean = false;

  constructor() {
    this.gstKit = new GstKit();
  }

  initialize(streamKey: string): void {
    if (this.pipelineInitialized) return;

    // Simple pipeline that receives RTMP data and processes it in memory
    // Using appsrc to receive buffers and appsink to pull them back
    const pipelineString = `
      appsrc name=src format=time is-live=true do-timestamp=true !
      video/x-h264,stream-format=avc,alignment=au !
      h264parse !
      appsink name=sink emit-signals=true sync=false
    `;

    try {
      this.gstKit.setPipeline(pipelineString);
      this.gstKit.play();
      this.pipelineInitialized = true;
      logger.info(`GStreamer pipeline initialized for stream: ${streamKey}`);
    } catch (error) {
      logger.error("Failed to initialize GStreamer pipeline", error);
    }
  }

  processVideoBuffer(buffer: Buffer): void {
    if (!this.pipelineInitialized) return;

    try {
      // Push video buffer to GStreamer pipeline
      this.gstKit.pushSample("src", buffer);
      logger.log(`Processed video buffer: ${buffer.length} bytes`);
    } catch (error) {
      logger.error("Failed to process video buffer", error);
    }
  }

  processAudioBuffer(buffer: Buffer): void {
    if (!this.pipelineInitialized) return;

    try {
      // Push audio buffer to GStreamer pipeline
      this.gstKit.pushSample("src", buffer);
      logger.log(`Processed audio buffer: ${buffer.length} bytes`);
    } catch (error) {
      logger.error("Failed to process audio buffer", error);
    }
  }

  pullProcessedFrame(): Buffer | null {
    if (!this.pipelineInitialized) return null;

    try {
      const frame = this.gstKit.pullSample("sink");
      if (frame) {
        logger.log(`Pulled processed frame: ${frame.length} bytes`);
        return frame;
      }
      return null;
    } catch (error) {
      logger.error("Failed to pull processed frame", error);
      return null;
    }
  }

  cleanup(): void {
    if (this.pipelineInitialized) {
      this.gstKit.stop();
      this.gstKit.cleanup();
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
      this.processRTMPMessages();
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
    while (this.buffer.length > 0) {
      const startLen = this.buffer.length;

      if (this.buffer.length < 1) break;

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
        if (this.buffer.length < headerSize) break;

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
        if (this.buffer.length < headerSize) break;

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
        if (this.buffer.length < headerSize) break;

        const timestampDelta = this.buffer.readUIntBE(offset, 3);
        timestamp += timestampDelta;
        this.lastTimestamp.set(csid, timestamp);
      }

      const incomplete = this.incompleteMessages.get(csid);
      const remainingBytes = incomplete
        ? incomplete.totalLength - incomplete.bytesReceived
        : messageLength;

      const bytesToRead = Math.min(remainingBytes, this.peerChunkSize);

      if (this.buffer.length < headerSize + bytesToRead) break;

      const chunkData = Buffer.from(
        this.buffer.subarray(headerSize, headerSize + bytesToRead),
      );
      this.buffer = this.buffer.subarray(headerSize + bytesToRead);

      if (!incomplete) {
        if (messageLength <= bytesToRead) {
          this.handleCompleteMessage(messageType, chunkData, csid, streamId);
        } else {
          this.incompleteMessages.set(csid, {
            buffer: chunkData,
            bytesReceived: bytesToRead,
            totalLength: messageLength,
            messageType,
            timestamp,
            streamId,
          });
        }
      } else {
        incomplete.buffer = Buffer.concat([incomplete.buffer, chunkData]);
        incomplete.bytesReceived += bytesToRead;

        if (incomplete.bytesReceived >= incomplete.totalLength) {
          this.handleCompleteMessage(
            incomplete.messageType,
            incomplete.buffer,
            csid,
            incomplete.streamId,
          );
          this.incompleteMessages.delete(csid);
        }
      }

      if (this.bytesReceived - this.lastAckSent >= this.windowAckSize) {
        this.sendAck(this.bytesReceived);
        this.lastAckSent = this.bytesReceived;
      }

      if (this.buffer.length === startLen) break;
    }
  }

  private async handleCompleteMessage(
    messageType: number,
    payload: Buffer,
    csid: number,
    streamId: number,
  ) {
    switch (messageType) {
      case MSG_SET_CHUNK_SIZE:
        if (payload.length >= 4) {
          this.peerChunkSize = payload.readUInt32BE(0) & 0x7fffffff;
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

      case MSG_AMF0_CMD:
      case MSG_AMF3_CMD:
        this.handleCommand(
          payload,
          csid,
          streamId,
          messageType === MSG_AMF3_CMD,
        );
        break;

      case MSG_AUDIO:
        logger.log(`Audio data: ${payload.length} bytes`);
        streamProcessor.processAudioBuffer(payload);
        break;

      case MSG_VIDEO:
        logger.log(`Video data: ${payload.length} bytes`);
        streamProcessor.processVideoBuffer(payload);
        break;

      default:
        logger.log(`Unhandled message type: ${messageType}`);
    }
  }

  private sendAck(bytes: number) {
    const payload = Buffer.alloc(4);
    payload.writeUInt32BE(bytes, 0);
    this.sendControlMessage(2, MSG_ACK, payload);
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
          this.sendCommandResponse(csid, "_result", transactionId, null, 1);
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
    const streamBegin = Buffer.alloc(6);
    streamBegin.writeUInt16BE(0, 0);
    streamBegin.writeUInt32BE(0, 2);
    this.sendControlMessage(2, MSG_USER_CONTROL, streamBegin);

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

    // Initialize GStreamer pipeline for this stream
    streamProcessor.initialize(streamKey);

    this.sendCommandResponse(csid, "onStatus", 0, null, {
      level: "status",
      code: "NetStream.Publish.Start",
      description: "Stream is now published",
      details: streamKey,
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

            conn.handleData(receivedData);
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

      logger.info(`Server started on rtmp://localhost:${this.port}`);
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
