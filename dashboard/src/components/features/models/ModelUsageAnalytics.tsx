import React, { useState, useMemo } from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo, ModelMetrics } from '../../../types/models';

interface UsageDataPoint {
  timestamp: string;
  requests: number;
  tokens_generated: number;
  average_latency: number;
  error_rate: number;
}

interface ModelUsageAnalyticsProps {
  model: ModelInfo;
  metrics?: ModelMetrics;
  usageData?: UsageDataPoint[];
  isLoading?: boolean;
  className?: string;
}

export const ModelUsageAnalytics: React.FC<ModelUsageAnalyticsProps> = ({
  model,
  metrics,
  usageData = [],
  isLoading = false,
  className = '',
}) => {
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d' | '30d'>('24h');
  const [selectedMetric, setSelectedMetric] = useState<'requests' | 'tokens' | 'latency' | 'errors'>('requests');

  // Generate mock usage data for demonstration
  const mockData = useMemo(() => {
    const now = new Date();
    const points: UsageDataPoint[] = [];
    const intervals = timeRange === '1h' ? 60 : timeRange === '6h' ? 72 : timeRange === '24h' ? 144 : timeRange === '7d' ? 168 : 720;
    
    for (let i = intervals; i >= 0; i--) {
      const timestamp = new Date(now.getTime() - i * (
        timeRange === '1h' ? 60000 : 
        timeRange === '6h' ? 300000 : 
        timeRange === '24h' ? 600000 : 
        timeRange === '7d' ? 3600000 : 
        3600000 * 24
      ));
      
      points.push({
        timestamp: timestamp.toISOString(),
        requests: Math.floor(Math.random() * 50 + 10 + Math.sin(i / 10) * 20),
        tokens_generated: Math.floor(Math.random() * 2000 + 500 + Math.cos(i / 8) * 500),
        average_latency: 50 + Math.random() * 100 + Math.sin(i / 15) * 30,
        error_rate: Math.random() * 5 + Math.sin(i / 20) * 2,
      });
    }
    return points;
  }, [timeRange]);

  const data = usageData.length > 0 ? usageData : mockData;

  // Calculate analytics
  const analytics = useMemo(() => {
    if (data.length === 0) return null;

    const totalRequests = data.reduce((sum, point) => sum + point.requests, 0);
    const totalTokens = data.reduce((sum, point) => sum + point.tokens_generated, 0);
    const avgLatency = data.reduce((sum, point) => sum + point.average_latency, 0) / data.length;
    const avgErrorRate = data.reduce((sum, point) => sum + point.error_rate, 0) / data.length;
    
    const peakRequests = Math.max(...data.map(p => p.requests));
    const peakTokens = Math.max(...data.map(p => p.tokens_generated));
    
    return {
      totalRequests,
      totalTokens,
      avgLatency,
      avgErrorRate,
      peakRequests,
      peakTokens,
      requestsPerHour: totalRequests / (data.length / (timeRange === '1h' ? 60 : timeRange === '6h' ? 10 : timeRange === '24h' ? 6 : timeRange === '7d' ? 1 : 0.25)),
      tokensPerSecond: totalTokens / (data.length * (timeRange === '1h' ? 60 : timeRange === '6h' ? 300 : timeRange === '24h' ? 600 : timeRange === '7d' ? 3600 : 86400)),
    };
  }, [data, timeRange]);

  const getMetricConfig = (metric: typeof selectedMetric) => {
    switch (metric) {
      case 'requests':
        return {
          title: 'Requests',
          color: 'rgb(59, 130, 246)',
          getValue: (point: UsageDataPoint) => point.requests,
          format: (value: number) => value.toString(),
        };
      case 'tokens':
        return {
          title: 'Tokens Generated',
          color: 'rgb(16, 185, 129)',
          getValue: (point: UsageDataPoint) => point.tokens_generated,
          format: (value: number) => value.toLocaleString(),
        };
      case 'latency':
        return {
          title: 'Average Latency',
          color: 'rgb(245, 158, 11)',
          getValue: (point: UsageDataPoint) => point.average_latency,
          format: (value: number) => `${value.toFixed(1)}ms`,
        };
      case 'errors':
        return {
          title: 'Error Rate',
          color: 'rgb(239, 68, 68)',
          getValue: (point: UsageDataPoint) => point.error_rate,
          format: (value: number) => `${value.toFixed(1)}%`,
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
          <span className="ml-2 text-gray-600 dark:text-gray-400">Loading analytics...</span>
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
              Usage Analytics
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
          </div>
          <div className="flex space-x-2">
            <Select
              value={selectedMetric}
              onChange={(value: string) => setSelectedMetric(value as any)}
              className="w-32"
            >
              <option value="requests">Requests</option>
              <option value="tokens">Tokens</option>
              <option value="latency">Latency</option>
              <option value="errors">Errors</option>
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
              <option value="30d">30d</option>
            </Select>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {model.status !== 'loaded' ? (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <p className="text-lg">Model not loaded</p>
            <p className="text-sm mt-1">Load the model to see usage analytics</p>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Key Metrics */}
            {analytics && (
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="text-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                    {analytics.totalRequests.toLocaleString()}
                  </div>
                  <div className="text-sm text-blue-700 dark:text-blue-300">Total Requests</div>
                </div>
                <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
                  <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                    {analytics.totalTokens.toLocaleString()}
                  </div>
                  <div className="text-sm text-green-700 dark:text-green-300">Total Tokens</div>
                </div>
                <div className="text-center p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
                  <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                    {analytics.avgLatency.toFixed(1)}ms
                  </div>
                  <div className="text-sm text-yellow-700 dark:text-yellow-300">Avg Latency</div>
                </div>
                <div className="text-center p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
                  <div className="text-2xl font-bold text-red-600 dark:text-red-400">
                    {analytics.avgErrorRate.toFixed(1)}%
                  </div>
                  <div className="text-sm text-red-700 dark:text-red-300">Error Rate</div>
                </div>
              </div>
            )}

            {/* Usage Chart */}
            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                {config.title} Over Time
              </h4>
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
                        r="2"
                        fill={config.color}
                        className="hover:r-3 transition-all cursor-pointer"
                      >
                        <title>
                          {new Date(point.timestamp).toLocaleString()}: {config.format(config.getValue(point))}
                        </title>
                      </circle>
                    );
                  })}
                </svg>

                {/* Y-axis labels */}
                <div className="absolute left-0 top-0 h-full flex flex-col justify-between text-xs text-gray-500 dark:text-gray-400 -ml-12">
                  <span>{config.format(maxValue)}</span>
                  <span>{config.format((maxValue + minValue) / 2)}</span>
                  <span>{config.format(minValue)}</span>
                </div>
              </div>
            </div>

            {/* Detailed Statistics */}
            {analytics && (
              <div>
                <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                  Performance Statistics
                </h4>
                <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Requests/Hour:</span>
                    <span className="font-medium">{analytics.requestsPerHour.toFixed(1)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Tokens/Second:</span>
                    <span className="font-medium">{analytics.tokensPerSecond.toFixed(1)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Peak Requests:</span>
                    <span className="font-medium">{analytics.peakRequests}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Peak Tokens:</span>
                    <span className="font-medium">{analytics.peakTokens.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Uptime:</span>
                    <span className="font-medium">
                      {model.loaded_at 
                        ? `${Math.floor((Date.now() - new Date(model.loaded_at).getTime()) / 3600000)}h`
                        : 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600 dark:text-gray-400">Efficiency:</span>
                    <span className="font-medium">
                      {metrics ? (metrics.tokens_per_second / (metrics.memory_usage / 1024 / 1024 / 1024)).toFixed(2) : 'N/A'} t/s/GB
                    </span>
                  </div>
                </div>
              </div>
            )}

            {/* Usage Insights */}
            {analytics && (
              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                <h4 className="font-medium text-blue-900 dark:text-blue-100 mb-2">Usage Insights</h4>
                <div className="space-y-1 text-sm text-blue-800 dark:text-blue-200">
                  {analytics.avgErrorRate > 5 && (
                    <p>• High error rate detected - consider model optimization</p>
                  )}
                  {analytics.avgLatency > 200 && (
                    <p>• High latency detected - consider hardware upgrade or quantization</p>
                  )}
                  {analytics.requestsPerHour > 1000 && (
                    <p>• High usage volume - monitor resource utilization</p>
                  )}
                  {analytics.avgErrorRate < 1 && analytics.avgLatency < 100 && (
                    <p>• Model performing optimally with low latency and error rates</p>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
};