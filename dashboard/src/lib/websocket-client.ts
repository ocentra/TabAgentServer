/**
 * WebSocket connection states
 */
export const WebSocketState = {
  CONNECTING: 'connecting',
  CONNECTED: 'connected',
  DISCONNECTED: 'disconnected',
  ERROR: 'error',
  RECONNECTING: 'reconnecting',
} as const;

export type WebSocketState = typeof WebSocketState[keyof typeof WebSocketState];

/**
 * WebSocket message types
 */
export interface WebSocketMessage<T = unknown> {
  type: string;
  data: T;
  timestamp: string;
}

/**
 * WebSocket event handlers
 */
export interface WebSocketEventHandlers<T = unknown> {
  onOpen?: (event: Event) => void;
  onMessage?: (message: WebSocketMessage<T>) => void;
  onError?: (error: Event) => void;
  onClose?: (event: CloseEvent) => void;
  onStateChange?: (state: WebSocketState) => void;
}

/**
 * WebSocket client configuration
 */
export interface WebSocketClientConfig {
  url: string;
  protocols?: string | string[];
  reconnectAttempts?: number;
  reconnectDelay?: number;
  maxReconnectDelay?: number;
  heartbeatInterval?: number;
  messageQueueSize?: number;
}

/**
 * WebSocket client for managing real-time connections
 */
export class WebSocketClient<T = unknown> {
  private ws: WebSocket | null = null;
  private config: Required<WebSocketClientConfig>;
  private state: WebSocketState = WebSocketState.DISCONNECTED;
  private reconnectCount = 0;
  private reconnectTimer: number | null = null;
  private heartbeatTimer: number | null = null;
  private messageQueue: WebSocketMessage<T>[] = [];
  private eventHandlers: WebSocketEventHandlers<T> = {};

  constructor(config: WebSocketClientConfig) {
    this.config = {
      protocols: [],
      reconnectAttempts: 5,
      reconnectDelay: 1000,
      maxReconnectDelay: 30000,
      heartbeatInterval: 30000,
      messageQueueSize: 100,
      ...config,
    };
  }

  /**
   * Connect to WebSocket server
   */
  connect(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      console.warn('WebSocket is already connected');
      return;
    }

    this.setState(WebSocketState.CONNECTING);

    try {
      this.ws = new WebSocket(this.config.url, this.config.protocols);
      this.setupEventListeners();
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      this.setState(WebSocketState.ERROR);
      this.scheduleReconnect();
    }
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    this.clearTimers();
    
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
    
    this.setState(WebSocketState.DISCONNECTED);
    this.reconnectCount = 0;
  }

  /**
   * Send message to server
   */
  send(message: WebSocketMessage<T>): boolean {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message));
        return true;
      } catch (error) {
        console.error('Failed to send WebSocket message:', error);
        return false;
      }
    } else {
      // Queue message if not connected
      this.queueMessage(message);
      return false;
    }
  }

  /**
   * Send raw data to server
   */
  sendRaw(data: string | ArrayBuffer | Blob): boolean {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(data);
        return true;
      } catch (error) {
        console.error('Failed to send raw WebSocket data:', error);
        return false;
      }
    }
    return false;
  }

  /**
   * Get current connection state
   */
  getState(): WebSocketState {
    return this.state;
  }

  /**
   * Check if WebSocket is connected
   */
  isConnected(): boolean {
    return this.state === WebSocketState.CONNECTED;
  }

  /**
   * Set event handlers
   */
  setEventHandlers(handlers: WebSocketEventHandlers<T>): void {
    this.eventHandlers = { ...this.eventHandlers, ...handlers };
  }

  /**
   * Remove event handlers
   */
  removeEventHandlers(): void {
    this.eventHandlers = {};
  }

  /**
   * Setup WebSocket event listeners
   */
  private setupEventListeners(): void {
    if (!this.ws) return;

    this.ws.onopen = (event) => {
      console.log('WebSocket connected:', this.config.url);
      this.setState(WebSocketState.CONNECTED);
      this.reconnectCount = 0;
      this.startHeartbeat();
      this.processMessageQueue();
      this.eventHandlers.onOpen?.(event);
    };

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage<T> = JSON.parse(event.data);
        this.eventHandlers.onMessage?.(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
        // Handle raw messages
        const rawMessage: WebSocketMessage<T> = {
          type: 'raw',
          data: event.data as T,
          timestamp: new Date().toISOString(),
        };
        this.eventHandlers.onMessage?.(rawMessage);
      }
    };

    this.ws.onerror = (event) => {
      console.error('WebSocket error:', event);
      this.setState(WebSocketState.ERROR);
      this.eventHandlers.onError?.(event);
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket closed:', event.code, event.reason);
      this.setState(WebSocketState.DISCONNECTED);
      this.clearTimers();
      this.eventHandlers.onClose?.(event);

      // Attempt reconnection if not a clean close
      if (event.code !== 1000 && this.reconnectCount < this.config.reconnectAttempts) {
        this.scheduleReconnect();
      }
    };
  }

  /**
   * Set connection state and notify handlers
   */
  private setState(newState: WebSocketState): void {
    if (this.state !== newState) {
      this.state = newState;
      this.eventHandlers.onStateChange?.(newState);
    }
  }

  /**
   * Schedule reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectCount >= this.config.reconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.setState(WebSocketState.RECONNECTING);
    
    const delay = Math.min(
      this.config.reconnectDelay * Math.pow(2, this.reconnectCount),
      this.config.maxReconnectDelay
    );

    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectCount + 1}/${this.config.reconnectAttempts})`);

    this.reconnectTimer = setTimeout(() => {
      this.reconnectCount++;
      this.connect();
    }, delay);
  }

  /**
   * Start heartbeat to keep connection alive
   */
  private startHeartbeat(): void {
    this.heartbeatTimer = setInterval(() => {
      if (this.isConnected()) {
        this.send({
          type: 'ping',
          data: {} as T,
          timestamp: new Date().toISOString(),
        });
      }
    }, this.config.heartbeatInterval);
  }

  /**
   * Clear all timers
   */
  private clearTimers(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  /**
   * Queue message for later sending
   */
  private queueMessage(message: WebSocketMessage<T>): void {
    this.messageQueue.push(message);
    
    // Limit queue size
    if (this.messageQueue.length > this.config.messageQueueSize) {
      this.messageQueue.shift();
    }
  }

  /**
   * Process queued messages
   */
  private processMessageQueue(): void {
    while (this.messageQueue.length > 0 && this.isConnected()) {
      const message = this.messageQueue.shift();
      if (message) {
        this.send(message);
      }
    }
  }
}

/**
 * WebSocket manager for handling multiple connections
 */
export class WebSocketManager {
  private connections = new Map<string, WebSocketClient<any>>();
  private baseUrl: string;

  constructor(baseUrl = 'ws://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  /**
   * Create or get WebSocket connection
   */
  getConnection<T = unknown>(
    path: string,
    config?: Partial<WebSocketClientConfig>
  ): WebSocketClient<T> {
    const key = path;
    
    if (!this.connections.has(key)) {
      const client = new WebSocketClient<T>({
        url: `${this.baseUrl}${path}`,
        ...config,
      });
      
      this.connections.set(key, client);
    }
    
    return this.connections.get(key) as WebSocketClient<T>;
  }

  /**
   * Remove connection
   */
  removeConnection(path: string): void {
    const client = this.connections.get(path);
    if (client) {
      client.disconnect();
      this.connections.delete(path);
    }
  }

  /**
   * Disconnect all connections
   */
  disconnectAll(): void {
    for (const [, client] of this.connections) {
      client.disconnect();
    }
    this.connections.clear();
  }

  /**
   * Get all active connections
   */
  getActiveConnections(): string[] {
    return Array.from(this.connections.keys()).filter(path => {
      const client = this.connections.get(path);
      return client?.isConnected();
    });
  }
}

// Create default WebSocket manager instance
export const wsManager = new WebSocketManager();