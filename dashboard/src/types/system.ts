// System monitoring types
export interface SystemStats {
  cpu: {
    usage_percent: number;
    cores: number;
    frequency: number;
  };
  memory: {
    total: number;
    used: number;
    available: number;
    usage_percent: number;
  };
  gpu?: {
    name: string;
    memory_total: number;
    memory_used: number;
    usage_percent: number;
    temperature?: number;
  };
  disk: {
    total: number;
    used: number;
    available: number;
    usage_percent: number;
  };
  network: {
    bytes_sent: number;
    bytes_received: number;
    connections: number;
  };
}

export interface ServerStatus {
  mode: 'native' | 'http' | 'webrtc' | 'web' | 'both' | 'all';
  ports: {
    http?: number;
    webrtc?: number;
  };
  uptime: number;
  version: string;
}