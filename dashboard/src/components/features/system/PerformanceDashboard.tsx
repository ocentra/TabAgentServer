import React, { useState } from 'react';
import { Card } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { ResourceCharts } from './ResourceCharts';
import { MetricCard, PerformanceMetricCard } from './MetricCard';
import { useSystemStats, usePerformanceStats, useModelMetrics } from '../../../hooks/useApi';

interface PerformanceDashboardProps {
  className?: string;
}

export const PerformanceDashboard: React.FC<PerformanceDashboardProps> = ({ 
  className = '' 
}) => {
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h'>('1h');
  const { data: systemStats, isLoading: systemLoading } = useSystemStats();
  const { data: performanceStats, isLoading: perfLoading } = usePerformanceStats();
  const { data: modelMetrics, isLoading: modelLoading } = useModelMetrics();

  const isLoading = systemLoading || perfLoading || modelLoading;

  if (isLoading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <Card className="p-6">
          <div className="flex items-center justify-center h-48">
            <LoadingSpinner size="lg" />
          </div>
        </Card>
      </div>
    );
  }

  const timeRangeOptions = [
    { value: '1h', label: '1 Hour' },
    { value: '6h', label: '6 Hours' },
    { value: '24h', label: '24 Hours' }
  ];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Performance Metrics
        </h2>
        <div className="flex space-x-2">
          {timeRangeOptions.map((option) => (
            <Button
              key={option.value}
              variant={timeRange === option.value ? 'default' : 'secondary'}
              size="sm"
              onClick={() => setTimeRange(option.value as any)}
            >
              {option.label}
            </Button>
          ))}
        </div>
      </div>

      {/* Key Performance Indicators */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <PerformanceMetricCard
          title="Response Time"
          value={performanceStats?.average_response_time || 0}
          unit="ms"
          target={100}
        />
        
        <PerformanceMetricCard
          title="Requests/Second"
          value={performanceStats?.requests_per_second || 0}
          unit="req/s"
          target={10}
        />
        
        <PerformanceMetricCard
          title="Error Rate"
          value={performanceStats?.error_rate || 0}
          unit="%"
          target={5}
        />
        
        <MetricCard
          title="Active Connections"
          value={systemStats?.active_connections || 0}
          unit="connections"
          status={
            (systemStats?.active_connections || 0) > 100 
              ? 'warning' 
              : (systemStats?.active_connections || 0) > 0 
              ? 'good' 
              : 'neutral'
          }
        />
      </div>

      {/* Resource Usage Charts */}
      <ResourceCharts timeRange={timeRange} />

      {/* Inference Performance */}
      {performanceStats?.inference_stats && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            AI Inference Performance
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <MetricCard
              title="Total Inferences"
              value={performanceStats.inference_stats.total_inferences}
              unit="requests"
              status="good"
            />
            
            <PerformanceMetricCard
              title="Time to First Token"
              value={performanceStats.inference_stats.average_ttft || 0}
              unit="ms"
              target={500}
            />
            
            <PerformanceMetricCard
              title="Tokens per Second"
              value={performanceStats.inference_stats.average_tokens_per_second || 0}
              unit="tokens/s"
              target={20}
            />
          </div>
        </Card>
      )}

      {/* Model Performance Breakdown */}
      {modelMetrics && Object.keys(modelMetrics).length > 0 && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            Model Performance
          </h3>
          <div className="space-y-4">
            {Object.entries(modelMetrics).map(([modelId, metrics]: [string, any]) => (
              <div key={modelId} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                <h4 className="font-medium text-gray-900 dark:text-white mb-3">
                  {modelId}
                </h4>
                <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
                  <div className="bg-gray-50 dark:bg-gray-800 rounded p-3">
                    <div className="text-sm text-gray-600 dark:text-gray-400">Inferences</div>
                    <div className="text-lg font-semibold text-gray-900 dark:text-white">
                      {metrics.inference_count || 0}
                    </div>
                  </div>
                  
                  <div className="bg-gray-50 dark:bg-gray-800 rounded p-3">
                    <div className="text-sm text-gray-600 dark:text-gray-400">Avg Latency</div>
                    <div className="text-lg font-semibold text-gray-900 dark:text-white">
                      {(metrics.average_latency || 0).toFixed(0)}ms
                    </div>
                  </div>
                  
                  <div className="bg-gray-50 dark:bg-gray-800 rounded p-3">
                    <div className="text-sm text-gray-600 dark:text-gray-400">Tokens/sec</div>
                    <div className="text-lg font-semibold text-gray-900 dark:text-white">
                      {(metrics.tokens_per_second || 0).toFixed(1)}
                    </div>
                  </div>
                  
                  <div className="bg-gray-50 dark:bg-gray-800 rounded p-3">
                    <div className="text-sm text-gray-600 dark:text-gray-400">Memory</div>
                    <div className="text-lg font-semibold text-gray-900 dark:text-white">
                      {((metrics.memory_usage || 0) / 1024 / 1024).toFixed(0)}MB
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}

      {/* System Health Summary */}
      <Card className="p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          System Health Summary
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-3">Resource Usage</h4>
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600 dark:text-gray-400">CPU</span>
                <div className="flex items-center space-x-2">
                  <div className="w-24 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                    <div 
                      className={`h-2 rounded-full ${
                        (systemStats?.cpu_usage || 0) > 80 
                          ? 'bg-red-500' 
                          : (systemStats?.cpu_usage || 0) > 60 
                          ? 'bg-yellow-500' 
                          : 'bg-green-500'
                      }`}
                      style={{ width: `${Math.min(systemStats?.cpu_usage || 0, 100)}%` }}
                    />
                  </div>
                  <span className="text-sm font-medium text-gray-900 dark:text-white">
                    {(systemStats?.cpu_usage || 0).toFixed(1)}%
                  </span>
                </div>
              </div>
              
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600 dark:text-gray-400">Memory</span>
                <div className="flex items-center space-x-2">
                  <div className="w-24 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                    <div 
                      className={`h-2 rounded-full ${
                        (systemStats?.memory_usage || 0) > 80 
                          ? 'bg-red-500' 
                          : (systemStats?.memory_usage || 0) > 60 
                          ? 'bg-yellow-500' 
                          : 'bg-green-500'
                      }`}
                      style={{ width: `${Math.min(systemStats?.memory_usage || 0, 100)}%` }}
                    />
                  </div>
                  <span className="text-sm font-medium text-gray-900 dark:text-white">
                    {(systemStats?.memory_usage || 0).toFixed(1)}%
                  </span>
                </div>
              </div>
              
              {systemStats?.gpu_usage !== undefined && (
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600 dark:text-gray-400">GPU</span>
                  <div className="flex items-center space-x-2">
                    <div className="w-24 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                      <div 
                        className={`h-2 rounded-full ${
                          systemStats.gpu_usage > 80 
                            ? 'bg-red-500' 
                            : systemStats.gpu_usage > 60 
                            ? 'bg-yellow-500' 
                            : 'bg-green-500'
                        }`}
                        style={{ width: `${Math.min(systemStats.gpu_usage, 100)}%` }}
                      />
                    </div>
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      {systemStats.gpu_usage.toFixed(1)}%
                    </span>
                  </div>
                </div>
              )}
            </div>
          </div>
          
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-3">Performance Status</h4>
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600 dark:text-gray-400">Response Time</span>
                <span className={`text-sm font-medium ${
                  (performanceStats?.average_response_time || 0) > 1000 
                    ? 'text-red-600 dark:text-red-400'
                    : (performanceStats?.average_response_time || 0) > 500
                    ? 'text-yellow-600 dark:text-yellow-400'
                    : 'text-green-600 dark:text-green-400'
                }`}>
                  {(performanceStats?.average_response_time || 0) < 1000 
                    ? 'Good' 
                    : (performanceStats?.average_response_time || 0) < 2000 
                    ? 'Fair' 
                    : 'Poor'}
                </span>
              </div>
              
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600 dark:text-gray-400">Error Rate</span>
                <span className={`text-sm font-medium ${
                  (performanceStats?.error_rate || 0) > 5 
                    ? 'text-red-600 dark:text-red-400'
                    : (performanceStats?.error_rate || 0) > 1
                    ? 'text-yellow-600 dark:text-yellow-400'
                    : 'text-green-600 dark:text-green-400'
                }`}>
                  {(performanceStats?.error_rate || 0) < 1 
                    ? 'Excellent' 
                    : (performanceStats?.error_rate || 0) < 5 
                    ? 'Good' 
                    : 'High'}
                </span>
              </div>
              
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600 dark:text-gray-400">Throughput</span>
                <span className={`text-sm font-medium ${
                  (performanceStats?.requests_per_second || 0) > 50 
                    ? 'text-green-600 dark:text-green-400'
                    : (performanceStats?.requests_per_second || 0) > 10
                    ? 'text-yellow-600 dark:text-yellow-400'
                    : 'text-red-600 dark:text-red-400'
                }`}>
                  {(performanceStats?.requests_per_second || 0) > 50 
                    ? 'High' 
                    : (performanceStats?.requests_per_second || 0) > 10 
                    ? 'Medium' 
                    : 'Low'}
                </span>
              </div>
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
};