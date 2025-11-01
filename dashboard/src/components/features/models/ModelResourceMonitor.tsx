import React from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo, ModelMetrics } from '../../../types/models';

interface ResourceUsage {
  cpu_usage: number;
  memory_usage: number;
  gpu_usage?: number;
  gpu_memory_usage?: number;
  disk_io?: number;
  network_io?: number;
}

interface ModelResourceMonitorProps {
  model: ModelInfo;
  metrics?: ModelMetrics;
  systemResources?: ResourceUsage;
  isLoading?: boolean;
  className?: string;
}

export const ModelResourceMonitor: React.FC<ModelResourceMonitorProps> = ({
  model,
  metrics,
  systemResources,
  isLoading = false,
  className = '',
}) => {
  const formatBytes = (bytes?: number) => {
    if (!bytes) return 'N/A';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const getUsageColor = (percentage: number) => {
    if (percentage < 50) return 'text-green-600 dark:text-green-400';
    if (percentage < 80) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
  };

  const getUsageBarColor = (percentage: number) => {
    if (percentage < 50) return 'bg-green-500';
    if (percentage < 80) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  if (isLoading) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center h-48">
          <LoadingSpinner size="lg" />
          <span className="ml-2 text-gray-600 dark:text-gray-400">Loading resources...</span>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Resource Utilization
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
      </CardHeader>

      <CardContent>
        {model.status !== 'loaded' ? (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <p className="text-lg">Model not loaded</p>
            <p className="text-sm mt-1">Load the model to monitor resource usage</p>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Memory Usage */}
            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Model Memory
                </span>
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  {formatBytes(metrics?.memory_usage)}
                </span>
              </div>
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                <div
                  className="bg-blue-500 h-3 rounded-full transition-all duration-300"
                  style={{ 
                    width: `${metrics?.memory_usage ? Math.min(100, (metrics.memory_usage / (8 * 1024 * 1024 * 1024)) * 100) : 0}%` 
                  }}
                />
              </div>
              <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
                <span>0 GB</span>
                <span>8 GB</span>
              </div>
            </div>

            {/* GPU Utilization */}
            {metrics?.gpu_utilization !== undefined && (
              <div>
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    GPU Utilization
                  </span>
                  <span className={`text-sm font-medium ${getUsageColor(metrics.gpu_utilization)}`}>
                    {metrics.gpu_utilization.toFixed(1)}%
                  </span>
                </div>
                <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                  <div
                    className={`h-3 rounded-full transition-all duration-300 ${getUsageBarColor(metrics.gpu_utilization)}`}
                    style={{ width: `${Math.min(100, Math.max(0, metrics.gpu_utilization))}%` }}
                  />
                </div>
              </div>
            )}

            {/* System Resources (if available) */}
            {systemResources && (
              <>
                <div className="border-t pt-4">
                  <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                    System Impact
                  </h4>
                  
                  {/* CPU Usage */}
                  <div className="mb-4">
                    <div className="flex justify-between items-center mb-2">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        CPU Usage
                      </span>
                      <span className={`text-sm font-medium ${getUsageColor(systemResources.cpu_usage)}`}>
                        {systemResources.cpu_usage.toFixed(1)}%
                      </span>
                    </div>
                    <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full transition-all duration-300 ${getUsageBarColor(systemResources.cpu_usage)}`}
                        style={{ width: `${Math.min(100, Math.max(0, systemResources.cpu_usage))}%` }}
                      />
                    </div>
                  </div>

                  {/* System Memory */}
                  <div className="mb-4">
                    <div className="flex justify-between items-center mb-2">
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        System Memory
                      </span>
                      <span className={`text-sm font-medium ${getUsageColor(systemResources.memory_usage)}`}>
                        {systemResources.memory_usage.toFixed(1)}%
                      </span>
                    </div>
                    <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full transition-all duration-300 ${getUsageBarColor(systemResources.memory_usage)}`}
                        style={{ width: `${Math.min(100, Math.max(0, systemResources.memory_usage))}%` }}
                      />
                    </div>
                  </div>

                  {/* GPU System Usage */}
                  {systemResources.gpu_usage !== undefined && (
                    <div className="mb-4">
                      <div className="flex justify-between items-center mb-2">
                        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                          System GPU
                        </span>
                        <span className={`text-sm font-medium ${getUsageColor(systemResources.gpu_usage)}`}>
                          {systemResources.gpu_usage.toFixed(1)}%
                        </span>
                      </div>
                      <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full transition-all duration-300 ${getUsageBarColor(systemResources.gpu_usage)}`}
                          style={{ width: `${Math.min(100, Math.max(0, systemResources.gpu_usage))}%` }}
                        />
                      </div>
                    </div>
                  )}
                </div>
              </>
            )}

            {/* Resource Statistics */}
            <div className="border-t pt-4">
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                Resource Statistics
              </h4>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Model Size:</span>
                  <span className="font-medium">
                    {model.parameters ? `${(model.parameters / 1e9).toFixed(1)}B params` : 'Unknown'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Quantization:</span>
                  <span className="font-medium">{model.quantization || 'None'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Memory Efficiency:</span>
                  <span className="font-medium">
                    {metrics?.memory_usage && model.parameters
                      ? `${((model.parameters * 2) / metrics.memory_usage * 100).toFixed(1)}%`
                      : 'N/A'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Inference Count:</span>
                  <span className="font-medium">
                    {metrics?.inference_count?.toLocaleString() || '0'}
                  </span>
                </div>
              </div>
            </div>

            {/* Resource Alerts */}
            {systemResources && (
              <div className="space-y-2">
                {systemResources.memory_usage > 90 && (
                  <div className="flex items-center space-x-2 text-red-600 dark:text-red-400 text-sm">
                    <span>⚠️</span>
                    <span>High memory usage detected</span>
                  </div>
                )}
                {systemResources.gpu_usage && systemResources.gpu_usage > 95 && (
                  <div className="flex items-center space-x-2 text-red-600 dark:text-red-400 text-sm">
                    <span>⚠️</span>
                    <span>GPU utilization is very high</span>
                  </div>
                )}
                {metrics?.tokens_per_second && metrics.tokens_per_second < 10 && (
                  <div className="flex items-center space-x-2 text-yellow-600 dark:text-yellow-400 text-sm">
                    <span>⚠️</span>
                    <span>Low throughput detected</span>
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
};