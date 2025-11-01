import React from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo, ModelMetrics as ModelMetricsType } from '../../../types/models';

interface ModelMetricsProps {
  model: ModelInfo;
  metrics?: ModelMetricsType;
  isLoading?: boolean;
  className?: string;
}

export const ModelMetrics: React.FC<ModelMetricsProps> = ({
  model,
  metrics,
  isLoading = false,
  className = '',
}) => {
  const formatNumber = (num?: number) => {
    if (!num) return 'N/A';
    if (num >= 1e9) return `${(num / 1e9).toFixed(1)}B`;
    if (num >= 1e6) return `${(num / 1e6).toFixed(1)}M`;
    if (num >= 1e3) return `${(num / 1e3).toFixed(1)}K`;
    return num.toString();
  };

  const formatBytes = (bytes?: number) => {
    if (!bytes) return 'N/A';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatDuration = (ms?: number) => {
    if (!ms) return 'N/A';
    if (ms < 1000) return `${ms.toFixed(1)}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  if (isLoading) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center h-32">
          <LoadingSpinner size="lg" />
          <span className="ml-2 text-gray-600 dark:text-gray-400">Loading metrics...</span>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Performance Metrics
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
      </CardHeader>

      <CardContent>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {/* Inference Count */}
          <div className="text-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
              {formatNumber(metrics?.inference_count)}
            </div>
            <div className="text-sm text-blue-700 dark:text-blue-300">Inferences</div>
          </div>

          {/* Average Latency */}
          <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {formatDuration(metrics?.average_latency)}
            </div>
            <div className="text-sm text-green-700 dark:text-green-300">Avg Latency</div>
          </div>

          {/* Tokens per Second */}
          <div className="text-center p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
            <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
              {metrics?.tokens_per_second?.toFixed(1) || 'N/A'}
            </div>
            <div className="text-sm text-yellow-700 dark:text-yellow-300">Tokens/sec</div>
          </div>

          {/* Memory Usage */}
          <div className="text-center p-4 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
              {formatBytes(metrics?.memory_usage)}
            </div>
            <div className="text-sm text-purple-700 dark:text-purple-300">Memory</div>
          </div>
        </div>

        {/* Additional Metrics */}
        {metrics && (
          <div className="mt-6 space-y-4">
            {/* GPU Utilization */}
            {metrics.gpu_utilization !== undefined && (
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-gray-600 dark:text-gray-400">GPU Utilization</span>
                  <span className="font-medium">{metrics.gpu_utilization.toFixed(1)}%</span>
                </div>
                <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                  <div
                    className="bg-gradient-to-r from-green-500 to-blue-500 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${Math.min(100, Math.max(0, metrics.gpu_utilization))}%` }}
                  />
                </div>
              </div>
            )}

            {/* Last Inference */}
            {metrics.last_inference && (
              <div className="flex justify-between text-sm">
                <span className="text-gray-600 dark:text-gray-400">Last Inference:</span>
                <span className="font-medium">
                  {new Date(metrics.last_inference).toLocaleString()}
                </span>
              </div>
            )}

            {/* Performance Status */}
            <div className="flex justify-between text-sm">
              <span className="text-gray-600 dark:text-gray-400">Status:</span>
              <span className={`font-medium ${
                model.status === 'loaded' ? 'text-green-600 dark:text-green-400' : 
                'text-gray-600 dark:text-gray-400'
              }`}>
                {model.status === 'loaded' ? 'Active' : 'Inactive'}
              </span>
            </div>
          </div>
        )}

        {/* No Metrics Available */}
        {!metrics && model.status === 'loaded' && (
          <div className="mt-6 text-center text-gray-500 dark:text-gray-400">
            <p>No performance metrics available</p>
            <p className="text-sm mt-1">Metrics will appear after the first inference</p>
          </div>
        )}

        {model.status !== 'loaded' && (
          <div className="mt-6 text-center text-gray-500 dark:text-gray-400">
            <p>Model not loaded</p>
            <p className="text-sm mt-1">Load the model to see performance metrics</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
};