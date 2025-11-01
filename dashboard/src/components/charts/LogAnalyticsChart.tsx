import React, { useState, useEffect } from 'react';
import { BarChart, PieChart, LineChart } from './index';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Select } from '@/components/ui/Select';
import { Badge } from '@/components/ui/Badge';
import { apiClient } from '@/lib/api-client';
import { formatNumber, formatDate } from '@/lib/utils';

export interface LogAnalyticsChartProps {
  className?: string;
}

interface LogStats {
  level: string;
  count: number;
  percentage: number;
}

interface LogTrend {
  timestamp: string;
  error: number;
  warn: number;
  info: number;
  debug: number;
  total: number;
}

interface LogSource {
  source: string;
  count: number;
  error_rate: number;
}

export const LogAnalyticsChart: React.FC<LogAnalyticsChartProps> = ({
  className = '',
}) => {
  const [logStats, setLogStats] = useState<LogStats[]>([]);
  const [logTrends, setLogTrends] = useState<LogTrend[]>([]);
  const [logSources, setLogSources] = useState<LogSource[]>([]);
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('24h');
  const [chartView, setChartView] = useState<'levels' | 'trends' | 'sources'>('levels');
  const [isLoading, setIsLoading] = useState(false);

  const fetchLogAnalytics = async () => {
    setIsLoading(true);
    try {
      const stats = await apiClient.getLogStats();
      
      // Transform stats to our format (mock data for development)
      const mockStats: LogStats[] = [
        { level: 'ERROR', count: 45, percentage: 5.2 },
        { level: 'WARN', count: 123, percentage: 14.1 },
        { level: 'INFO', count: 567, percentage: 65.3 },
        { level: 'DEBUG', count: 134, percentage: 15.4 },
      ];
      
      const mockTrends: LogTrend[] = Array.from({ length: 24 }, (_, i) => ({
        timestamp: new Date(Date.now() - (23 - i) * 60 * 60 * 1000).toISOString(),
        error: Math.floor(Math.random() * 10),
        warn: Math.floor(Math.random() * 25),
        info: Math.floor(Math.random() * 100) + 20,
        debug: Math.floor(Math.random() * 30),
        total: 0, // Will be calculated
      }));
      
      // Calculate totals
      mockTrends.forEach(trend => {
        trend.total = trend.error + trend.warn + trend.info + trend.debug;
      });
      
      const mockSources: LogSource[] = [
        { source: 'HTTP Server', count: 234, error_rate: 2.1 },
        { source: 'WebRTC', count: 156, error_rate: 1.8 },
        { source: 'Model Engine', count: 189, error_rate: 3.2 },
        { source: 'Database', count: 98, error_rate: 0.5 },
        { source: 'Native Messaging', count: 67, error_rate: 1.2 },
      ];
      
      setLogStats(mockStats);
      setLogTrends(mockTrends);
      setLogSources(mockSources);
    } catch (error) {
      console.error('Failed to fetch log analytics:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchLogAnalytics();
  }, [timeRange]);

  // Chart data transformations
  const levelDistributionData = logStats.map(stat => ({
    name: stat.level,
    value: stat.count,
    color: getLevelColor(stat.level),
  }));

  const trendData = logTrends.map(trend => ({
    timestamp: new Date(trend.timestamp).getTime(),
    ERROR: trend.error,
    WARN: trend.warn,
    INFO: trend.info,
    DEBUG: trend.debug,
    Total: trend.total,
  }));

  const sourceData = logSources.map(source => ({
    name: source.source,
    count: source.count,
    error_rate: source.error_rate,
  }));

  function getLevelColor(level: string): string {
    switch (level.toUpperCase()) {
      case 'ERROR': return '#ef4444';
      case 'WARN': return '#f59e0b';
      case 'INFO': return '#3b82f6';
      case 'DEBUG': return '#6b7280';
      default: return '#8b5cf6';
    }
  }

  const trendSeries = [
    { key: 'ERROR', name: 'Error', color: '#ef4444' },
    { key: 'WARN', name: 'Warning', color: '#f59e0b' },
    { key: 'INFO', name: 'Info', color: '#3b82f6' },
    { key: 'DEBUG', name: 'Debug', color: '#6b7280' },
  ];

  const sourceSeries = [
    { key: 'count', name: 'Log Count', color: '#3b82f6' },
  ];

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Controls */}
      <Card>
        <CardHeader>
          <CardTitle>Log Analytics Dashboard</CardTitle>
          <CardDescription>
            Analyze log patterns, error rates, and system behavior trends
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
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
            
            <div>
              <label className="block text-sm font-medium mb-2">Chart View</label>
              <Select
                value={chartView}
                onChange={(value) => setChartView(value as any)}
              >
                <option value="levels">Log Level Distribution</option>
                <option value="trends">Log Trends Over Time</option>
                <option value="sources">Log Sources Analysis</option>
              </Select>
            </div>
            
            <div className="flex items-end">
              <Button onClick={fetchLogAnalytics} disabled={isLoading}>
                {isLoading ? 'Loading...' : 'Refresh'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        {logStats.map((stat) => (
          <Card key={stat.level}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-2xl font-bold" style={{ color: getLevelColor(stat.level) }}>
                    {formatNumber(stat.count)}
                  </div>
                  <div className="text-sm text-muted-foreground">{stat.level} Logs</div>
                </div>
                <Badge 
                  variant={stat.level === 'ERROR' ? 'error' : stat.level === 'WARN' ? 'warning' : 'secondary'}
                >
                  {stat.percentage.toFixed(1)}%
                </Badge>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Main Chart */}
      <Card>
        <CardHeader>
          <CardTitle>
            {chartView === 'levels' && 'Log Level Distribution'}
            {chartView === 'trends' && 'Log Volume Trends'}
            {chartView === 'sources' && 'Log Sources Analysis'}
          </CardTitle>
          <CardDescription>
            {chartView === 'levels' && 'Distribution of log entries by severity level'}
            {chartView === 'trends' && 'Log volume patterns over time by severity'}
            {chartView === 'sources' && 'Log volume and error rates by source component'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {chartView === 'levels' && (
            <PieChart
              data={levelDistributionData}
              height={400}
              showLabels={true}
              innerRadius={60}
              centerLabel={{
                value: logStats.reduce((sum, stat) => sum + stat.count, 0),
                label: 'Total Logs'
              }}
              valueFormatter={formatNumber}
            />
          )}
          
          {chartView === 'trends' && (
            <LineChart
              data={trendData}
              series={trendSeries}
              height={400}
              xAxisFormatter={(value) => formatDate(value).split(' ')[1]}
              yAxisFormatter={formatNumber}
              tooltipFormatter={(value, name) => [formatNumber(value), name]}
            />
          )}
          
          {chartView === 'sources' && (
            <BarChart
              data={sourceData}
              series={sourceSeries}
              height={400}
              layout="vertical"
              tooltipFormatter={(value, name) => [formatNumber(value), name]}
            />
          )}
        </CardContent>
      </Card>

      {/* Error Rate Analysis */}
      {chartView === 'sources' && (
        <Card>
          <CardHeader>
            <CardTitle>Error Rate by Source</CardTitle>
            <CardDescription>
              Error rates across different system components
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {logSources.map((source) => (
                <div key={source.source} className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <div className="font-medium">{source.source}</div>
                    <div className="text-sm text-muted-foreground">
                      {formatNumber(source.count)} total logs
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`text-lg font-bold ${
                      source.error_rate > 5 ? 'text-red-600' :
                      source.error_rate > 2 ? 'text-yellow-600' : 'text-green-600'
                    }`}>
                      {source.error_rate.toFixed(1)}%
                    </div>
                    <div className="text-sm text-muted-foreground">Error Rate</div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Recent Patterns */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Log Patterns</CardTitle>
          <CardDescription>
            Key insights from recent log activity
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <h4 className="font-medium">Highest Activity</h4>
              <div className="text-sm text-muted-foreground">
                Peak log volume: {formatDate(new Date())} with {formatNumber(Math.max(...logTrends.map(t => t.total)))} entries
              </div>
            </div>
            
            <div className="space-y-2">
              <h4 className="font-medium">Error Trends</h4>
              <div className="text-sm text-muted-foreground">
                {logTrends.slice(-2)[1]?.error > logTrends.slice(-2)[0]?.error 
                  ? '📈 Error rate increasing' 
                  : '📉 Error rate stable/decreasing'
                }
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};