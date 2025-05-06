export interface WSConfig {
  ws_url: string;
}

export type WSMessage =
  | {
      ok: true;
      //username
      from: string;
      total_amount: number;
      //mb url for image
      invoice_payload: string;
      message: string;
    }
  | {
      ok: false;
      error: string;
    };

//ws has automatic pong message sending, so there is no need to handle ping messages.
export class WSClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private pingInterval: number | null = null;
  private lastPongTime = 0;
  private readonly config: WSConfig;
  private messageCallback: ((message: WSMessage) => void) | null = null;
  private lastPingTime = 0;
  //for sending ping message
  private pingTimeout: number | null = null;
  //for
  private readonly PING_TIMEOUT = 8000;

  constructor(config: WSConfig) {
    this.config = config;
  }

  public onMessage(callback: (message: WSMessage) => void) {
    this.messageCallback = callback;
  }

  public connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.config.ws_url);

        this.ws.onopen = () => {
          console.log("WebSocket connected");
          this.reconnectDelay = 1000;
          this.reconnectAttempts = 0;
          // this.startPingInterval();
          resolve();
        };

        this.ws.onclose = (event) => {
          // console.log("WebSocket closed:", event.code, event.reason);
          // this.stopPingInterval();
          this.handleReconnect();
        };

        this.ws.onerror = (error) => {
          // console.error("WebSocket error:", error);
          // this.stopPingInterval();
          reject(error);
        };

        this.ws.onmessage = (message) => {
          try {
            if (message.data === "ping") {
              if (this.ws?.readyState === WebSocket.OPEN) {
                // this.ws.send("pong");
              }
              return;
            }

            const data = JSON.parse(message.data);
            this.handleMessage(data);
          } catch (error) {
            console.error("Error parsing message:", error);
          }
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  // private startPingInterval() {
  //   this.pingInterval = window.setInterval(() => {
  //     if (this.ws?.readyState === WebSocket.OPEN) {
  //       this.ws.send(JSON.stringify({ type: "ping" }));
  //     }
  //   }, 6000);
  // }

  // private stopPingInterval() {
  //   if (this.pingInterval) {
  //     clearInterval(this.pingInterval);
  //     this.pingInterval = null;
  //   }
  // }

  private handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(
        `Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`
      );

      setTimeout(() => {
        this.connect().catch((error) => {
          console.error("Reconnection failed:", error);
        });
      }, this.reconnectDelay);
      this.reconnectDelay *= 2;
    } else {
      console.error("Max reconnection attempts reached");
    }
  }

  private handleMessage(data: WSMessage) {
    // console.log("Received message:", data);
    if (this.messageCallback) {
      this.messageCallback(data);
    }
  }

  public send(message: string) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      // this.ws.send(message);
    } else {
      console.error("WebSocket is not connected");
    }
  }

  public disconnect() {
    // this.stopPingInterval();
    if (this.ws) {
      this.ws.close(1000, "Client initiated disconnect");
      this.ws = null;
    }
  }
}
