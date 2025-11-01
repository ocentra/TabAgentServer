import React from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { Badge } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo, ModelMetrics } from '../../../types/models';

interface ModelCardProps {
  model: ModelInfo;
  metrics?: ModelMetrics;
  onLoad?: () => void;
  onUnload?: () => void;
  onConfigure?: () => void;
  isLoading?: boolean;
}

export const ModelCard: React.FC<ModelCardProps> = ({
  model,
  metrics,
  onLoad,
  onUnload,
  onConfigure,
  isLoading = false,
}) => {
  const getStatusColor = (status: ModelInfo['status']) => {
    switch (status) {
      case 'loaded':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'loading':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'unloaded':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
      case 'error':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
    }
  };

  const getTypeIcon = (type: ModelInfo['type']) => {
    switch (type) {
      case 'language':
        return '💬';
      case 'vision':
        return '👁️';
      case 'audio':
        return '🎵';
      case 'multimodal':
        return '🔄';
      default:
        return '🤖';
    }
  };

  const formatBytes = (bytes?: number) => {
    if (!bytes) return 'N/A';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatNumber = (num?: number) => {
    if (!num) return 'N/A';
    if (num >= 1e9) return `${(num / 1e9).toFixed(1)}B`;
    if (num >= 1e6) return `${(num / 1e6).toFixed(1)}M`;
    if (num >= 1e3) return `${(num / 1e3).toFixed(1)}K`;
    return num.toString();
  };

  return (
    <Card className="h-full">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center space-x-2">
            <span className="text-2xl">{getTypeIcon(model.type)}</span>
            <div>
              <h3 className="font-semibold text-lg text-gray-900 dark:text-white">
                {model.name}
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {model.id}
              </p>
            </div>
          </div>
          <Badge className={getStatusColor(model.status)}>
            {model.status === 'loading' && (
              <LoadingSpinner size="sm" className="mr-1" />
            )}
            {model.status}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Model Metadata */}
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-gray-500 dark:text-gray-400">Type:</span>
            <p className="font-medium capitalize">{model.type}</p>
          </div>
          <div>
            <span className="text-gray-500 dark:text-gray-400">Parameters:</span>
            <p className="font-medium">{formatNumber(model.parameters)}</p>
          </div>
          <div>
            <span className="text-gray-500 dark:text-gray-400">Memory:</span>
            <p className="font-medium">{formatBytes(model.memory_usage)}</p>
          </div>
          <div>
            <span className="text-gray-500 dark:text-gray-400">Quantization:</span>
            <p className="font-medium">{model.quantization || 'None'}</p>
          </div>
        </div>

        {/* Performance Metrics (if loaded) */}
        {model.status === 'loaded' && metrics && (
          <div className="border-t pt-4">
            <h4 className="font-medium text-sm text-gray-700 dark:text-gray-300 mb-2">
              Performance Metrics
            </h4>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div>
                <span className="text-gray-500 dark:text-gray-400">Inferences:</span>
                <p className="font-medium">{formatNumber(metrics.inference_count)}</p>
              </div>
              <div>
                <span className="text-gray-500 dark:text-gray-400">Avg Latency:</span>
                <p className="font-medium">{metrics.average_latency.toFixed(1)}ms</p>
              </div>
              <div>
                <span className="text-gray-500 dark:text-gray-400">Tokens/sec:</span>
                <p className="font-medium">{metrics.tokens_per_second.toFixed(1)}</p>
              </div>
              <div>
                <span className="text-gray-500 dark:text-gray-400">GPU Usage:</span>
                <p className="font-medium">
                  {metrics.gpu_utilization ? `${metrics.gpu_utilization.toFixed(1)}%` : 'N/A'}
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Loading Progress */}
        {model.status === 'loading' && (
          <div className="border-t pt-4">
            <div className="flex items-center space-x-2">
              <LoadingSpinner size="sm" />
              <span className="text-sm text-gray-600 dark:text-gray-400">
                Loading model...
              </span>
            </div>
          </div>
        )}

        {/* Error State */}
        {model.status === 'error' && (
          <div className="border-t pt-4">
            <div className="flex items-center space-x-2 text-red-600 dark:text-red-400">
              <span className="text-sm">⚠️ Model failed to load</span>
            </div>
          </div>
        )}

        {/* Action Buttons */}
        <div className="border-t pt-4 flex space-x-2">
          {model.status === 'unloaded' && (
            <Button
              onClick={onLoad}
              disabled={isLoading}
              size="sm"
              className="flex-1"
            >
              {isLoading ? <LoadingSpinner size="sm" /> : 'Load'}
            </Button>
          )}
          
          {model.status === 'loaded' && (
            <Button
              onClick={onUnload}
              disabled={isLoading}
              variant="outline"
              size="sm"
              className="flex-1"
            >
              {isLoading ? <LoadingSpinner size="sm" /> : 'Unload'}
            </Button>
          )}
          
          {model.status === 'error' && (
            <Button
              onClick={onLoad}
              disabled={isLoading}
              size="sm"
              className="flex-1"
            >
              {isLoading ? <LoadingSpinner size="sm" /> : 'Retry'}
            </Button>
          )}
          
          <Button
            onClick={onConfigure}
            variant="outline"
            size="sm"
            disabled={model.status === 'loading'}
          >
            Configure
          </Button>
        </div>

        {/* Loaded Time */}
        {model.loaded_at && (
          <div className="text-xs text-gray-500 dark:text-gray-400">
            Loaded: {new Date(model.loaded_at).toLocaleString()}
          </div>
        )}
      </CardContent>
    </Card>
  );
};