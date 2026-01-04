/**
 * Simple Logger Utility
 * Provides basic logging functionality with optional debug mode
 */

class Logger {
  private debugMode: boolean;
  private logPrefix: string;

  constructor(debugMode: boolean = false, prefix: string = "[RTMP]") {
    this.debugMode = debugMode;
    this.logPrefix = prefix;
  }

  setDebugMode(enabled: boolean): void {
    this.debugMode = enabled;
  }

  private formatMessage(message: string): string {
    const timestamp = new Date().toISOString();
    return `${this.logPrefix} [${timestamp}] ${message}`;
  }

  log(message: string): void {
    if (this.debugMode) {
      console.log(this.formatMessage(message));
    }
  }

  error(message: string, error?: any): void {
    console.error(this.formatMessage(message));
    if (error) {
      console.error(error);
    }
  }

  warn(message: string): void {
    if (this.debugMode) {
      console.warn(this.formatMessage(message));
    }
  }

  info(message: string): void {
    console.log(this.formatMessage(message));
  }
}

export { Logger };
