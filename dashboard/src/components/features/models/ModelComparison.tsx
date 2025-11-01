import React, { useState } from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo, ModelMetrics } from '../../../types/models';

interface ModelComparisonProps {
  models: ModelInfo[];
  metrics: Record<string, ModelMetrics>;
  isLoading?: boolean;
  className?: string;
}

export const ModelComparison: React.FC<ModelComparisonProps> = ({
  models,
  metrics,
  isLoading = false,
  className = '',
}) => {
  const [selectedModels, setSelectedModels] = useState<string[]>([]);
  const [comparisonMetric, setComparisonMetric] = useState<'latency' | 'throughput' | 'memory' | 'efficiency'>('latency');

  // Filter to loaded models only
  const loadedModels = models.filter(m => m.status === 'loaded');

  const handleModelToggle = (modelId: string) => {
    setSelectedModels(prev => {
      if (prev.includes(modelId)) {
        return prev.filter(id => id !== modelId);
      } else if (prev.length < 4) { // Limit to 4 models for comparison
        return [...prev, modelId];
      }
      return prev;
    });
  };

  const getComparisonValue = (modelId: string, metric: typeof comparisonMetric) => {
    const modelMetrics = metrics[modelId];
    const model = models.find(m => m.id === modelId);
    
    if (!modelMetrics || !model) return 0;

    switch (metric) {
      case 'latency':
        return modelMetrics.average_latency;
      case 'throughput':
        return modelMetrics.tokens_per_second;
      case 'memory':
        return modelMetrics.memory_usage / 1024 / 1024 / 1024; // Convert to GB
      case 'efficiency':
        // Efficiency = tokens per second per GB of memory
        return modelMetrics.tokens_per_second / (modelMetrics.memory_usage / 1024 / 1024 / 1024);
      default:
        return 0;
    }
  };

  const getMetricConfig = (metric: typeof comparisonMetric) => {
    switch (metric) {
      case 'latency':
        return {
          title: 'Average Latency',
          unit: 'ms',
          lowerIsBetter: true,
          format: (value: number) => `${value.toFixed(1)}ms`,
        };
      case 'throughput':
        return {
          title: 'Throughput',
          unit: 'tokens/sec',
          lowerIsBetter: false,
          format: (value: number) => `${value.toFixed(1)} t/s`,
        };
      case 'memory':
        return {
          title: 'Memory Usage',
          unit: 'GB',
          lowerIsBetter: true,
          format: (value: number) => `${value.toFixed(1)} GB`,
        };
      case 'efficiency':
        return {
          title: 'Efficiency',
          unit: 'tokens/sec/GB',
          lowerIsBetter: false,
          format: (value: number) => `${value.toFixed(2)} t/s/GB`,
        };
    }
  };

  const config = getMetricConfig(comparisonMetric);

  if (isLoading) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center h-64">
          <LoadingSpinner size="lg" />
          <span className="ml-2 text-gray-600 dark:text-gray-400">Loading comparison...</span>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Model Comparison
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Compare performance across loaded models
            </p>
          </div>
          <div className="flex space-x-2">
            <Select
              value={comparisonMetric}
              onChange={(value: string) => setComparisonMetric(value as any)}
              className="w-32"
            >
              <option value="latency">Latency</option>
              <option value="throughput">Throughput</option>
              <option value="memory">Memory</option>
              <option value="efficiency">Efficiency</option>
            </Select>
            <Button
              onClick={() => setSelectedModels([])}
              variant="outline"
              size="sm"
              disabled={selectedModels.length === 0}
            >
              Clear All
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {loadedModels.length === 0 ? (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <p className="text-lg">No loaded models</p>
            <p className="text-sm mt-1">Load some models to compare their performance</p>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Model Selection */}
            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                Select Models to Compare (max 4)
              </h4>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {loadedModels.map((model) => (
                  <div
                    key={model.id}
                    className={`p-3 border rounded-lg cursor-pointer transition-all ${
                      selectedModels.includes(model.id)
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                    }`}
                    onClick={() => handleModelToggle(model.id)}
                  >
                    <div className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        checked={selectedModels.includes(model.id)}
                        onChange={() => handleModelToggle(model.id)}
                        className="rounded"
                      />
                      <div className="flex-1 min-w-0">
                        <p className="font-medium text-sm truncate">{model.name}</p>
                        <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                          {model.type} • {model.parameters ? `${(model.parameters / 1e9).toFixed(1)}B` : 'Unknown'} params
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Comparison Results */}
            {selectedModels.length > 0 && (
              <div>
                <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                  {config.title} Comparison
                </h4>
                
                {/* Comparison Chart */}
                <div className="space-y-3">
                  {selectedModels.map((modelId) => {
                    const model = models.find(m => m.id === modelId);
                    const value = getComparisonValue(modelId, comparisonMetric);
                    const maxValue = Math.max(...selectedModels.map(id => getComparisonValue(id, comparisonMetric)));
                    const percentage = maxValue > 0 ? (value / maxValue) * 100 : 0;
                    
                    if (!model) return null;

                    return (
                      <div key={modelId} className="space-y-2">
                        <div className="flex items-center justify-between text-sm">
                          <span className="font-medium truncate flex-1 mr-2">{model.name}</span>
                          <span className="text-gray-600 dark:text-gray-400">
                            {config.format(value)}
                          </span>
                        </div>
                        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                          <div
                            className={`h-2 rounded-full transition-all duration-300 ${
                              config.lowerIsBetter
                                ? percentage === Math.min(...selectedModels.map(id => (getComparisonValue(id, comparisonMetric) / maxValue) * 100))
                                  ? 'bg-green-500'
                                  : 'bg-blue-500'
                                : percentage === 100
                                  ? 'bg-green-500'
                                  : 'bg-blue-500'
                            }`}
                            style={{ width: `${percentage}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>

                {/* Comparison Table */}
                <div className="mt-6 overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b border-gray-200 dark:border-gray-700">
                        <th className="text-left py-2 font-medium text-gray-900 dark:text-white">Model</th>
                        <th className="text-right py-2 font-medium text-gray-900 dark:text-white">Latency</th>
                        <th className="text-right py-2 font-medium text-gray-900 dark:text-white">Throughput</th>
                        <th className="text-right py-2 font-medium text-gray-900 dark:text-white">Memory</th>
                        <th className="text-right py-2 font-medium text-gray-900 dark:text-white">Efficiency</th>
                      </tr>
                    </thead>
                    <tbody>
                      {selectedModels.map((modelId) => {
                        const model = models.find(m => m.id === modelId);
                        const modelMetrics = metrics[modelId];
                        
                        if (!model || !modelMetrics) return null;

                        return (
                          <tr key={modelId} className="border-b border-gray-100 dark:border-gray-800">
                            <td className="py-2 font-medium">{model.name}</td>
                            <td className="py-2 text-right">{modelMetrics.average_latency.toFixed(1)}ms</td>
                            <td className="py-2 text-right">{modelMetrics.tokens_per_second.toFixed(1)} t/s</td>
                            <td className="py-2 text-right">{(modelMetrics.memory_usage / 1024 / 1024 / 1024).toFixed(1)} GB</td>
                            <td className="py-2 text-right">
                              {(modelMetrics.tokens_per_second / (modelMetrics.memory_usage / 1024 / 1024 / 1024)).toFixed(2)} t/s/GB
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>

                {/* Winner Badge */}
                {selectedModels.length > 1 && (
                  <div className="mt-4 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <span className="text-green-600 dark:text-green-400">🏆</span>
                      <span className="font-medium text-green-800 dark:text-green-200">
                        Best {config.title}: {
                          (() => {
                            const values = selectedModels.map(id => ({ id, value: getComparisonValue(id, comparisonMetric) }));
                            const best = config.lowerIsBetter 
                              ? values.reduce((min, curr) => curr.value < min.value ? curr : min)
                              : values.reduce((max, curr) => curr.value > max.value ? curr : max);
                            const model = models.find(m => m.id === best.id);
                            return model?.name || 'Unknown';
                          })()
                        }
                      </span>
                    </div>
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