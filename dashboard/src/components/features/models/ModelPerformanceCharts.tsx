import React, { useState, useMemo } from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo, ModelMetrics } from '../../../types/models';

interface PerformanceDataPoint {
  timestamp: string;
  latency: number;
  tokensPerSecond: number;
  memoryUsage: number;
  gpuUtilization?: number;
}

interface ModelPerformanceChartsProps {
  model: ModelInfo;
  metrics?: ModelMetrics;
  historicalData?: PerformanceDataPoint[];
  isLoading?: boolean;
  className?: string;
}

export const ModelPerformanceCharts: React.FC<ModelPerformanceChartsProps> = ({
  model,
  metrics,
  historicalData = [],
  isLoading = false,
  className = '',
}) => {
  const [selectedMetric, setSelectedMetric] = useState<'latency' | 'throughput' | 'memory' | 'gpu'>('latency');
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('1h');

  // Generate mock historical data for demonstration
  const mockData = useMemo(() => {
    const now = new Date();
    const points: PerformanceDataPoint[] = [];
    const intervals = timeRange === '1h' ? 60 : timeRange === '6h' ? 72 : timeRange === '24h' ? 144 : 168;
    
    for (let i = intervals; i >= 0; i--) {
      const timestamp = new Date(now.getTime() - i * (timeRange === '1h' ? 60000 : timeRange === '6h' ? 300000 : timeRange === '24h' ? 600000 : 3600000));
      points.push({
        timestamp: timestamp.toISOString(),
        latency: 50 + Math.random() * 100 + Math.sin(i / 10) * 20,
        tokensPerSecond: 40 + Math.random() * 20 + Math.cos(i / 8) * 10,
        memoryUsage: 2000000000 + Math.random() * 500000000,
        gpuUtilization: 60 + Math.random() * 30 + Math.sin(i / 15) * 15,
      });
    }
    return points;
  }, [timeRange]);

  const data = historicalData.length > 0 ? historicalData : mockData;

  const getMetricConfig = (metric: typeof selectedMetric) => {
    switch (metric) {
      case 'latency':
        return {
          title: 'Response Latency',
          unit: 'ms',
          color: 'rgb(59, 130, 246)',
          getValue: (point: PerformanceDataPoint) => point.latency,
        };
      case 'throughput':
        return {
          title: 'Throughput',
          unit: 'tokens/sec',
          color: 'rgb(16, 185, 129)',
          getValue: (point: PerformanceDataPoint) => point.tokensPerSecond,
        };
      case 'memory':
        return {
          title: 'Memory Usage',
          unit: 'GB',
          color: 'rgb(139, 92, 246)',
          getValue: (point: PerformanceDataPoint) => point.memoryUsage / 1024 / 1024 / 1024,
        };
      case 'gpu':
        return {
          title: 'GPU Utilization',
          unit: '%',
          color: 'rgb(245, 158, 11)',
          getValue: (point: PerformanceDataPoint) => point.gpuUtilization || 0,
        };
    }
  };

  const config = getMetricConfig(selectedMetric);
  const values = data.map(config.getValue);
  const minValue = Math.min(...values);
  const maxValue = Math.max(...values);
  const range = maxValue - minValue || 1;

  if (isLoading) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center h-64">
          <LoadingSpinner size="lg" />
          <span className="ml-2 text-gray-600 dark:text-gray-400">Loading charts...</span>
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
              Performance Trends
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
          </div>
          <div className="flex space-x-2">
            <Select
              value={selectedMetric}
              onChange={(value: string) => setSelectedMetric(value as any)}
              className="w-32"
            >
              <option value="latency">Latency</option>
              <option value="throughput">Throughput</option>
              <option value="memory">Memory</option>
              <option value="gpu">GPU Usage</option>
            </Select>
            <Select
              value={timeRange}
              onChange={(value: string) => setTimeRange(value as any)}
              className="w-20"
            >
              <option value="1h">1h</option>
              <option value="6h">6h</option>
              <option value="24h">24h</option>
              <option value="7d">7d</option>
            </Select>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {model.status !== 'loaded' ? (
          <div className="flex items-center justify-center h-64 text-gray-500 dark:text-gray-400">
            <div className="text-center">
              <p className="text-lg">Model not loaded</p>
              <p className="text-sm mt-1">Load the model to see performance charts</p>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {/* Chart Header */}
            <div className="flex items-center justify-between">
              <h4 className="font-medium text-gray-900 dark:text-white">
                {config.title}
              </h4>
              <div className="text-sm text-gray-500 dark:text-gray-400">
                Current: {metrics ? config.getValue({ 
                  timestamp: '', 
                  latency: metrics.average_latency, 
                  tokensPerSecond: metrics.tokens_per_second, 
                  memoryUsage: metrics.memory_usage,
                  gpuUtilization: metrics.gpu_utilization 
                }).toFixed(1) : 'N/A'} {config.unit}
              </div>
            </div>

            {/* Simple SVG Chart */}
            <div className="relative h-48 bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <svg width="100%" height="100%" className="overflow-visible">
                {/* Grid lines */}
                {[0, 25, 50, 75, 100].map((percent) => (
                  <line
                    key={percent}
                    x1="0"
                    y1={`${percent}%`}
                    x2="100%"
                    y2={`${percent}%`}
                    stroke="currentColor"
                    strokeWidth="1"
                    className="text-gray-300 dark:text-gray-600"
                    opacity="0.3"
                  />
                ))}

                {/* Chart line */}
                <polyline
                  fill="none"
                  stroke={config.color}
                  strokeWidth="2"
                  points={data.map((point, index) => {
                    const x = (index / (data.length - 1)) * 100;
                    const y = 100 - ((config.getValue(point) - minValue) / range) * 100;
                    return `${x},${y}`;
                  }).join(' ')}
                />

                {/* Data points */}
                {data.map((point, index) => {
                  const x = (index / (data.length - 1)) * 100;
                  const y = 100 - ((config.getValue(point) - minValue) / range) * 100;
                  return (
                    <circle
                      key={index}
                      cx={`${x}%`}
                      cy={`${y}%`}
                      r="3"
                      fill={config.color}
                      className="hover:r-4 transition-all cursor-pointer"
                    >
                      <title>
                        {new Date(point.timestamp).toLocaleTimeString()}: {config.getValue(point).toFixed(1)} {config.unit}
                      </title>
                    </circle>
                  );
                })}
              </svg>

              {/* Y-axis labels */}
              <div className="absolute left-0 top-0 h-full flex flex-col justify-between text-xs text-gray-500 dark:text-gray-400 -ml-8">
                <span>{maxValue.toFixed(1)}</span>
                <span>{((maxValue + minValue) / 2).toFixed(1)}</span>
                <span>{minValue.toFixed(1)}</span>
              </div>
            </div>

            {/* Statistics */}
            <div className="grid grid-cols-4 gap-4 text-sm">
              <div className="text-center">
                <div className="font-medium text-gray-900 dark:text-white">
                  {minValue.toFixed(1)}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Min</div>
              </div>
              <div className="text-center">
                <div className="font-medium text-gray-900 dark:text-white">
                  {maxValue.toFixed(1)}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Max</div>
              </div>
              <div className="text-center">
                <div className="font-medium text-gray-900 dark:text-white">
                  {(values.reduce((a, b) => a + b, 0) / values.length).toFixed(1)}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Avg</div>
              </div>
              <div className="text-center">
                <div className="font-medium text-gray-900 dark:text-white">
                  {data.length}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Points</div>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};