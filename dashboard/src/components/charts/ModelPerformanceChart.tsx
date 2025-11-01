import React, { useState, useEffect } from 'react';
import { LineChart, BarChart } from './index';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Select } from '@/components/ui/Select';
import { apiClient } from '@/lib/api-client';
import { formatDuration, formatNumber } from '@/lib/utils';

export interface ModelPerformanceChartProps {
  className?: string;
  modelId?: string;
}

interface ModelMetrics {
  model_id: string;
  model_name: string;
  inference_count: number;
  average_ttft: number; // Time to first token
  average_tokens_per_second: number;
  total_tokens: number;
  error_rate: number;
  memory_usage: number;
  timestamp: string;
}

export const ModelPerformanceChart: React.FC<ModelPerformanceChartProps> = ({
  className = '',
  modelId,
}) => {
  const [metrics, setMetrics] = useState<ModelMetrics[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>(modelId || 'all');
  const [chartType, setChartType] = useState<'performance' | 'comparison'>('performance');
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('1h');
  const [isLoading, setIsLoading] = useState(false);

  const fetchModelMetrics = async () => {
    setIsLoading(true);
    try {
      const data = await apiClient.getModelMetrics(selectedModel === 'all' ? undefined : selectedModel);
      
      // Transform API response to match our interface
      const transformedMetrics: ModelMetrics[] = Array.isArray(data) ? data : [data].filter(Boolean);
      setMetrics(transformedMetrics);
    } catch (error) {
      console.error('Failed to fetch model metrics:', error);
      // Generate mock data for development
      const mockMetrics: ModelMetrics[] = Array.from({ length: 20 }, (_, i) => ({
        model_id: selectedModel === 'all' ? `model-${i % 3}` : selectedModel,
        model_name: selectedModel === 'all' ? `Model ${i % 3 + 1}` : `Selected Model`,
        inference_count: Math.floor(Math.random() * 1000),
        average_ttft: Math.random() * 500 + 100,
        average_tokens_per_second: Math.random() * 50 + 10,
        total_tokens: Math.floor(Math.random() * 100000),
        error_rate: Math.random() * 5,
        memory_usage: Math.random() * 8000 + 1000,
        timestamp: new Date(Date.now() - i * 60000).toISOString(),
      }));
      setMetrics(mockMetrics);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchModelMetrics();
  }, [selectedModel, timeRange]);

  // Performance over time chart data
  const performanceData = metrics.map(m => ({
    timestamp: new Date(m.timestamp).getTime(),
    ttft: m.average_ttft,
    tokens_per_second: m.average_tokens_per_second,
    error_rate: m.error_rate,
    memory_usage: m.memory_usage / 1024, // Convert to GB
  }));

  // Model comparison chart data
  const comparisonData = metrics.reduce((acc, m) => {
    const existing = acc.find(item => item.name === m.model_name);
    if (existing) {
      existing.inference_count += m.inference_count;
      existing.total_tokens += m.total_tokens;
      existing.avg_ttft = (existing.avg_ttft + m.average_ttft) / 2;
      existing.avg_tps = (existing.avg_tps + m.average_tokens_per_second) / 2;
    } else {
      acc.push({
        name: m.model_name,
        inference_count: m.inference_count,
        total_tokens: m.total_tokens,
        avg_ttft: m.average_ttft,
        avg_tps: m.average_tokens_per_second,
      });
    }
    return acc;
  }, [] as any[]);

  const performanceSeries = [
    {
      key: 'ttft',
      name: 'Time to First Token (ms)',
      color: '#3b82f6',
    },
    {
      key: 'tokens_per_second',
      name: 'Tokens/Second',
      color: '#10b981',
    },
    {
      key: 'error_rate',
      name: 'Error Rate (%)',
      color: '#ef4444',
    },
  ];

  const comparisonSeries = [
    {
      key: 'inference_count',
      name: 'Inference Count',
      color: '#3b82f6',
    },
    {
      key: 'total_tokens',
      name: 'Total Tokens',
      color: '#10b981',
    },
  ];

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Controls */}
      <Card>
        <CardHeader>
          <CardTitle>Model Performance Analytics</CardTitle>
          <CardDescription>
            Monitor and compare model inference performance and resource usage
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Model</label>
              <Select
                value={selectedModel}
                onChange={setSelectedModel}
              >
                <option value="all">All Models</option>
                <option value="model-1">Model 1</option>
                <option value="model-2">Model 2</option>
                <option value="model-3">Model 3</option>
              </Select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">Chart Type</label>
              <Select
                value={chartType}
                onChange={(value) => setChartType(value as any)}
              >
                <option value="performance">Performance Over Time</option>
                <option value="comparison">Model Comparison</option>
              </Select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">Time Range</label>
              <Select
                value={timeRange}
                onChange={(value) => setTimeRange(value as any)}
              >
                <option value="1h">Last Hour</option>
                <option value="6h">Last 6 Hours</option>
                <option value="24h">Last 24 Hours</option>
                <option value="7d">Last 7 Days</option>
              </Select>
            </div>
            
            <div className="flex items-end">
              <Button onClick={fetchModelMetrics} disabled={isLoading}>
                {isLoading ? 'Loading...' : 'Refresh'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Performance Charts */}
      {chartType === 'performance' ? (
        <Card>
          <CardHeader>
            <CardTitle>Performance Metrics Over Time</CardTitle>
            <CardDescription>
              Track inference speed, latency, and error rates
            </CardDescription>
          </CardHeader>
          <CardContent>
            {performanceData.length > 0 ? (
              <LineChart
                data={performanceData}
                series={performanceSeries}
                height={400}
                tooltipFormatter={(value, name) => {
                  if (name.includes('Token')) {
                    return [formatDuration(value), name];
                  }
                  if (name.includes('Rate')) {
                    return [`${value.toFixed(2)}%`, name];
                  }
                  return [formatNumber(value), name];
                }}
              />
            ) : (
              <div className="flex items-center justify-center h-64 text-muted-foreground">
                <p>No performance data available</p>
              </div>
            )}
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>Model Comparison</CardTitle>
            <CardDescription>
              Compare inference counts and token generation across models
            </CardDescription>
          </CardHeader>
          <CardContent>
            {comparisonData.length > 0 ? (
              <BarChart
                data={comparisonData}
                series={comparisonSeries}
                height={400}
                layout="vertical"
                tooltipFormatter={(value, name) => [formatNumber(value), name]}
              />
            ) : (
              <div className="flex items-center justify-center h-64 text-muted-foreground">
                <p>No comparison data available</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-blue-600">
              {formatNumber(metrics.reduce((sum, m) => sum + m.inference_count, 0))}
            </div>
            <div className="text-sm text-muted-foreground">Total Inferences</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-green-600">
              {metrics.length > 0 
                ? formatDuration(metrics.reduce((sum, m) => sum + m.average_ttft, 0) / metrics.length)
                : '0ms'
              }
            </div>
            <div className="text-sm text-muted-foreground">Avg Time to First Token</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-purple-600">
              {metrics.length > 0 
                ? formatNumber(metrics.reduce((sum, m) => sum + m.average_tokens_per_second, 0) / metrics.length)
                : '0'
              }
            </div>
            <div className="text-sm text-muted-foreground">Avg Tokens/Second</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="text-2xl font-bold text-orange-600">
              {metrics.length > 0 
                ? `${(metrics.reduce((sum, m) => sum + m.error_rate, 0) / metrics.length).toFixed(2)}%`
                : '0%'
              }
            </div>
            <div className="text-sm text-muted-foreground">Avg Error Rate</div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};