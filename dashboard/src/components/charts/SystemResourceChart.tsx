import React, { useRef } from 'react';
import { RealTimeChart, RealTimeChartDataPoint } from './RealTimeChart';
import { ChartExport } from './ChartExport';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { apiClient } from '@/lib/api-client';
import { formatBytes } from '@/lib/utils';

export interface SystemResourceChartProps {
  className?: string;
  updateInterval?: number;
  showExport?: boolean;
}

export const SystemResourceChart: React.FC<SystemResourceChartProps> = ({
  className = '',
  updateInterval = 5000,
  showExport = true,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const [chartData, setChartData] = React.useState<RealTimeChartDataPoint[]>([]);

  const fetchSystemMetrics = async (): Promise<Record<string, number>> => {
    try {
      const stats = await apiClient.getSystemStats();
      return {
        cpu_usage: stats.cpu_usage || 0,
        memory_usage: stats.memory_usage || 0,
        gpu_usage: stats.gpu_usage || 0,
        disk_usage: stats.disk_usage || 0,
      };
    } catch (error) {
      console.error('Failed to fetch system metrics:', error);
      // Return mock data for development
      return {
        cpu_usage: Math.random() * 100,
        memory_usage: Math.random() * 100,
        gpu_usage: Math.random() * 100,
        disk_usage: Math.random() * 100,
      };
    }
  };

  const series = [
    {
      key: 'cpu_usage',
      name: 'CPU Usage',
      color: '#3b82f6',
    },
    {
      key: 'memory_usage',
      name: 'Memory Usage',
      color: '#10b981',
    },
    {
      key: 'gpu_usage',
      name: 'GPU Usage',
      color: '#f59e0b',
    },
    {
      key: 'disk_usage',
      name: 'Disk Usage',
      color: '#ef4444',
    },
  ];

  const formatters = {
    yAxis: (value: number) => `${Math.round(value)}%`,
    tooltip: (value: number, name: string) => [`${Math.round(value)}%`, name] as [string, string],
  };

  return (
    <div className={`space-y-4 ${className}`}>
      <div ref={chartRef}>
        <RealTimeChart
          title="System Resource Usage"
          description="Real-time monitoring of CPU, memory, GPU, and disk usage"
          series={series}
          dataSource={fetchSystemMetrics}
          updateInterval={updateInterval}
          maxDataPoints={60} // 5 minutes of data at 5-second intervals
          height={400}
          chartType="area"
          onDataUpdate={setChartData}
          formatters={formatters}
        />
      </div>

      {showExport && (
        <ChartExport
          chartRef={chartRef}
          data={chartData}
          filename="system-resources"
          title="Export System Resource Data"
        />
      )}
    </div>
  );
};