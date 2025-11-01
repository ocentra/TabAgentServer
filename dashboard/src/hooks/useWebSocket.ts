import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import {
    WebSocketClient,
    WebSocketState,
    wsManager,
} from '../lib/websocket-client';
import type {
    WebSocketMessage,
    WebSocketEventHandlers,
} from '../lib/websocket-client';

/**
 * WebSocket hook configuration
 */
export interface UseWebSocketConfig<T = unknown> {
    // Whether to connect automatically on mount
    autoConnect?: boolean;
    // Whether to reconnect on component remount
    reconnectOnMount?: boolean;
    // Custom event handlers
    onOpen?: (event: Event) => void;
    onMessage?: (message: WebSocketMessage<T>) => void;
    onError?: (error: Event) => void;
    onClose?: (event: CloseEvent) => void;
    // Message filter function
    messageFilter?: (message: WebSocketMessage<T>) => boolean;
}

/**
 * WebSocket hook return type
 */
export interface UseWebSocketReturn<T = unknown> {
    // Connection state
    state: WebSocketState;
    isConnected: boolean;

    // Connection controls
    connect: () => void;
    disconnect: () => void;

    // Message handling
    sendMessage: (message: WebSocketMessage<T>) => boolean;
    sendRaw: (data: string | ArrayBuffer | Blob) => boolean;

    // Last received message
    lastMessage: WebSocketMessage<T> | null;

    // Message history
    messageHistory: WebSocketMessage<T>[];
    clearHistory: () => void;
}

/**
 * React hook for WebSocket connections
 */
export function useWebSocket<T = unknown>(
    path: string,
    config: UseWebSocketConfig<T> = {}
): UseWebSocketReturn<T> {
    const {
        autoConnect = true,
        reconnectOnMount = true,
        onOpen,
        onMessage,
        onError,
        onClose,
        messageFilter,
    } = config;

    const [state, setState] = useState<WebSocketState>(WebSocketState.DISCONNECTED);
    const [lastMessage, setLastMessage] = useState<WebSocketMessage<T> | null>(null);
    const [messageHistory, setMessageHistory] = useState<WebSocketMessage<T>[]>([]);

    const clientRef = useRef<WebSocketClient<T> | null>(null);
    const mountedRef = useRef(true);

    // Initialize WebSocket client
    useEffect(() => {
        if (!clientRef.current) {
            clientRef.current = wsManager.getConnection<T>(path);
        }

        const client = clientRef.current;

        // Set up event handlers
        const handlers: WebSocketEventHandlers<T> = {
            onOpen: (event) => {
                if (mountedRef.current) {
                    setState(WebSocketState.CONNECTED);
                    onOpen?.(event);
                }
            },

            onMessage: (message) => {
                if (!mountedRef.current) return;

                // Apply message filter if provided
                if (messageFilter && !messageFilter(message)) {
                    return;
                }

                setLastMessage(message);
                setMessageHistory(prev => {
                    const newHistory = [...prev, message];
                    // Keep only last 100 messages to prevent memory issues
                    return newHistory.slice(-100);
                });

                onMessage?.(message);
            },

            onError: (error) => {
                if (mountedRef.current) {
                    setState(WebSocketState.ERROR);
                    onError?.(error);
                }
            },

            onClose: (event) => {
                if (mountedRef.current) {
                    setState(WebSocketState.DISCONNECTED);
                    onClose?.(event);
                }
            },

            onStateChange: (newState) => {
                if (mountedRef.current) {
                    setState(newState);
                }
            },
        };

        client.setEventHandlers(handlers);

        // Auto-connect if enabled
        if (autoConnect) {
            client.connect();
        }

        // Cleanup on unmount
        return () => {
            mountedRef.current = false;
            client.removeEventHandlers();

            // Don't disconnect here as other components might be using the same connection
            // The WebSocketManager handles connection lifecycle
        };
    }, [path, autoConnect, onOpen, onMessage, onError, onClose, messageFilter]);

    // Reconnect on mount if enabled
    useEffect(() => {
        if (reconnectOnMount && clientRef.current && !clientRef.current.isConnected()) {
            clientRef.current.connect();
        }
    }, [reconnectOnMount]);

    // Connection controls
    const connect = useCallback(() => {
        clientRef.current?.connect();
    }, []);

    const disconnect = useCallback(() => {
        clientRef.current?.disconnect();
    }, []);

    // Message sending
    const sendMessage = useCallback((message: WebSocketMessage<T>): boolean => {
        return clientRef.current?.send(message) ?? false;
    }, []);

    const sendRaw = useCallback((data: string | ArrayBuffer | Blob): boolean => {
        return clientRef.current?.sendRaw(data) ?? false;
    }, []);

    // Clear message history
    const clearHistory = useCallback(() => {
        setMessageHistory([]);
        setLastMessage(null);
    }, []);

    return {
        state,
        isConnected: state === WebSocketState.CONNECTED,
        connect,
        disconnect,
        sendMessage,
        sendRaw,
        lastMessage,
        messageHistory,
        clearHistory,
    };
}

/**
 * Hook for real-time log streaming
 */
export function useLogStream() {
    const { lastMessage, messageHistory, isConnected, ...rest } = useWebSocket('/ws/logs', {
        messageFilter: (message) => message.type === 'log',
    });

    const logs = messageHistory
        .filter(msg => msg.type === 'log')
        .map(msg => msg.data)
        .filter(Boolean);

    return {
        logs,
        lastLog: lastMessage?.type === 'log' ? lastMessage.data : null,
        isConnected,
        ...rest,
    };
}

/**
 * Hook for real-time system metrics
 */
export function useSystemMetricsStream() {
    const { lastMessage, isConnected, ...rest } = useWebSocket('/ws/system', {
        messageFilter: (message) => message.type === 'metrics',
    });

    return {
        metrics: lastMessage?.type === 'metrics' ? lastMessage.data : null,
        isConnected,
        ...rest,
    };
}

/**
 * Hook for real-time model status updates
 */
export function useModelStatusStream() {
    const { lastMessage, messageHistory, isConnected, ...rest } = useWebSocket('/ws/models', {
        messageFilter: (message) => ['model_loaded', 'model_unloaded', 'model_error'].includes(message.type),
    });

    const modelUpdates = messageHistory
        .filter(msg => ['model_loaded', 'model_unloaded', 'model_error'].includes(msg.type))
        .map(msg => ({
            type: msg.type,
            data: msg.data,
            timestamp: msg.timestamp,
        }));

    return {
        modelUpdates,
        lastUpdate: lastMessage ? {
            type: lastMessage.type,
            data: lastMessage.data,
            timestamp: lastMessage.timestamp,
        } : null,
        isConnected,
        ...rest,
    };
}

/**
 * Hook for managing multiple WebSocket connections
 */
export function useMultipleWebSockets<T = unknown>(
    connections: Array<{
        path: string;
        config?: UseWebSocketConfig<T>;
    }>
) {
    // Create individual hooks for each connection
    const results = useMemo(() => {
        const hookResults = [];
        for (const { path, config } of connections) {
            // eslint-disable-next-line react-hooks/rules-of-hooks
            hookResults.push(useWebSocket<T>(path, config));
        }
        return hookResults;
    }, [connections]);

    const allConnected = results.every(result => result.isConnected);
    const anyConnected = results.some(result => result.isConnected);

    const connectAll = useCallback(() => {
        results.forEach(result => result.connect());
    }, [results]);

    const disconnectAll = useCallback(() => {
        results.forEach(result => result.disconnect());
    }, [results]);

    return {
        connections: results,
        allConnected,
        anyConnected,
        connectAll,
        disconnectAll,
    };
}