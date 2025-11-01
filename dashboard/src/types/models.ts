// Model information types
export interface ModelInfo {
  id: string;
  name: string;
  type: 'language' | 'vision' | 'audio' | 'multimodal';
  status: 'loaded' | 'loading' | 'unloaded' | 'error';
  memory_usage?: number;
  parameters?: number;
  quantization?: string;
  loaded_at?: string;
}

export interface ModelMetrics {
  inference_count: number;
  average_latency: number;
  tokens_per_second: number;
  memory_usage: number;
  gpu_utilization?: number;
  last_inference?: string;
}

export interface ModelLoadRequest {
  model_id: string;
  config?: Record<string, unknown>;
}

export interface ModelUnloadRequest {
  model_id: string;
}