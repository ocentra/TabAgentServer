import React, { useState, useEffect } from 'react';
import {
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area
} from 'recharts';
import { Card } from '../../ui/Card';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { useSystemStats, usePerformanceStats } from '../../../hooks/useApi';

interface ResourceData {
  timestamp: string;
  cpu: number;
  memory: number;
  gpu?: number;
  disk: number;
}

interface ResourceChartsProps {
  className?: string;
  timeRange?: '1h' | '6h' | '24h';
}

export const ResourceCharts: React.FC<ResourceChartsProps> = ({ 
  className = '',
  timeRange: _timeRange = '1h'
}) => {
  const { data: systemStats, isLoading: systemLoading } = useSystemStats();
  const { data: performanceStats, isLoading: perfLoading } = usePerformanceStats();
  const [historicalData, setHistoricalData] = useState<ResourceData[]>([]);
  const [selectedMetric, setSelectedMetric] = useState<'cpu' | 'memory' | 'gpu' | 'disk'>('cpu');

  // Simulate historical data collection
  useEffect(() => {
    if (systemStats) {
      const newDataPoint: ResourceData = {
        timestamp: new Date().toLocaleTimeString(),
        cpu: systemStats.cpu_usage || 0,
        memory: systemStats.memory_usage || 0,
        gpu: systemStats.gpu_usage,
        disk: systemStats.disk_usage || 0,
      };

      setHistoricalData(prev => {
        const updated = [...prev, newDataPoint];
        // Keep only last 60 data points (for 1 hour at 1-minute intervals)
        return updated.slice(-60);
      });
    }
  }, [systemStats]);

  const isLoading = systemLoading || perfLoading;

  if (isLoading && historicalData.length === 0) {
    return (
      <Card className={`p-6 ${className}`}>
        <div className="flex items-center justify-center h-64">
          <LoadingSpinner size="lg" />
        </div>
      </Card>
    );
  }

  const formatPercentage = (value: number) => `${value.toFixed(1)}%`;

  const getMetricColor = (metric: string) => {
    switch (metric) {
      case 'cpu': return '#3B82F6'; // blue
      case 'memory': return '#10B981'; // green
      case 'gpu': return '#F59E0B'; // amber
      case 'disk': return '#EF4444'; // red
      default: return '#6B7280'; // gray
    }
  };

  const getCurrentValue = (metric: string) => {
    if (!systemStats) return 0;
    switch (metric) {
      case 'cpu': return systemStats.cpu_usage || 0;
      case 'memory': return systemStats.memory_usage || 0;
      case 'gpu': return systemStats.gpu_usage || 0;
      case 'disk': return systemStats.disk_usage || 0;
      default: return 0;
    }
  };

  const metrics = [
    { key: 'cpu', label: 'CPU Usage', unit: '%' },
    { key: 'memory', label: 'Memory Usage', unit: '%' },
    ...(systemStats?.gpu_usage !== undefined ? [{ key: 'gpu', label: 'GPU Usage', unit: '%' }] : []),
    { key: 'disk', label: 'Disk Usage', unit: '%' }
  ];

  return (
    <Card className={`p-6 ${className}`}>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Resource Usage
          </h3>
          <div className="flex space-x-2">
            {metrics.map((metric) => (
              <button
                key={metric.key}
                onClick={() => setSelectedMetric(metric.key as any)}
                className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${
                  selectedMetric === metric.key
                    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200'
                    : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200'
                }`}
              >
                {metric.label}
              </button>
            ))}
          </div>
        </div>

        {/* Current Values Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {metrics.map((metric) => {
            const value = getCurrentValue(metric.key);
            const isHigh = value > 80;
            const isMedium = value > 60;
            
            return (
              <div
                key={metric.key}
                className={`p-4 rounded-lg border-2 transition-colors ${
                  selectedMetric === metric.key
                    ? 'border-blue-200 bg-blue-50 dark:border-blue-700 dark:bg-blue-900/20'
                    : 'border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="text-sm text-gray-600 dark:text-gray-400">
                    {metric.label}
                  </div>
                  <div className={`w-3 h-3 rounded-full ${
                    isHigh ? 'bg-red-500' : isMedium ? 'bg-yellow-500' : 'bg-green-500'
                  }`} />
                </div>
                <div className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {formatPercentage(value)}
                </div>
              </div>
            );
          })}
        </div>

        {/* Chart */}
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={historicalData}>
              <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
              <XAxis 
                dataKey="timestamp" 
                tick={{ fontSize: 12 }}
                tickFormatter={(value) => {
                  const time = new Date(value).toLocaleTimeString();
                  return time.split(':').slice(0, 2).join(':');
                }}
              />
              <YAxis 
                domain={[0, 100]}
                tick={{ fontSize: 12 }}
                tickFormatter={formatPercentage}
              />
              <Tooltip
                formatter={(value: number) => [formatPercentage(value), selectedMetric.toUpperCase()]}
                labelFormatter={(label) => `Time: ${label}`}
                contentStyle={{
                  backgroundColor: 'rgba(255, 255, 255, 0.95)',
                  border: '1px solid #e5e7eb',
                  borderRadius: '8px',
                  boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)'
                }}
              />
              <Area
                type="monotone"
                dataKey={selectedMetric}
                stroke={getMetricColor(selectedMetric)}
                fill={getMetricColor(selectedMetric)}
                fillOpacity={0.2}
                strokeWidth={2}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Performance Stats */}
        {performanceStats && (
          <div className="border-t pt-4">
            <h4 className="text-md font-medium text-gray-900 dark:text-white mb-3">
              Performance Metrics
            </h4>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-3">
                <div className="text-sm text-gray-600 dark:text-gray-400">Requests/sec</div>
                <div className="text-lg font-semibold text-gray-900 dark:text-white">
                  {performanceStats.requests_per_second?.toFixed(1) || '0.0'}
                </div>
              </div>
              <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-3">
                <div className="text-sm text-gray-600 dark:text-gray-400">Avg Response Time</div>
                <div className="text-lg font-semibold text-gray-900 dark:text-white">
                  {performanceStats.average_response_time?.toFixed(0) || '0'}ms
                </div>
              </div>
              <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-3">
                <div className="text-sm text-gray-600 dark:text-gray-400">Error Rate</div>
                <div className="text-lg font-semibold text-gray-900 dark:text-white">
                  {formatPercentage(performanceStats.error_rate || 0)}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
};