import React, { useState, useEffect, useCallback, useRef } from 'react';
import { LineChart, LineChartDataPoint, LineChartSeries } from './LineChart';
import { AreaChart, AreaChartDataPoint, AreaChartSeries } from './AreaChart';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';

export interface RealTimeChartProps {
  title: string;
  description?: string;
  series: (LineChartSeries | AreaChartSeries)[];
  dataSource: () => Promise<Record<string, number>>;
  updateInterval?: number;
  maxDataPoints?: number;
  height?: number;
  chartType?: 'line' | 'area';
  showControls?: boolean;
  autoStart?: boolean;
  className?: string;
  onDataUpdate?: (data: RealTimeChartDataPoint[]) => void;
  formatters?: {
    xAxis?: (value: any) => string;
    yAxis?: (value: any) => string;
    tooltip?: (value: any, name: string) => [string, string];
  };
}

export interface RealTimeChartDataPoint extends LineChartDataPoint, AreaChartDataPoint {
  timestamp: number;
}

export const RealTimeChart: React.FC<RealTimeChartProps> = ({
  title,
  description,
  series,
  dataSource,
  updateInterval = 5000,
  maxDataPoints = 50,
  height = 300,
  chartType = 'line',
  showControls = true,
  autoStart = true,
  className = '',
  onDataUpdate,
  formatters,
}) => {
  const [data, setData] = useState<RealTimeChartDataPoint[]>([]);
  const [isRunning, setIsRunning] = useState(autoStart);
  const [isPaused, setIsPaused] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const dataBufferRef = useRef<RealTimeChartDataPoint[]>([]);

  // Fetch new data point
  const fetchDataPoint = useCallback(async () => {
    try {
      const newValues = await dataSource();
      const timestamp = Date.now();
      
      const newDataPoint: RealTimeChartDataPoint = {
        timestamp,
        ...newValues,
      };

      setData(prevData => {
        const updatedData = [...prevData, newDataPoint];
        
        // Keep only the most recent data points
        if (updatedData.length > maxDataPoints) {
          updatedData.splice(0, updatedData.length - maxDataPoints);
        }
        
        dataBufferRef.current = updatedData;
        onDataUpdate?.(updatedData);
        
        return updatedData;
      });

      setLastUpdate(new Date());
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch data');
      console.error('Real-time chart data fetch error:', err);
    }
  }, [dataSource, maxDataPoints, onDataUpdate]);

  // Start/stop data fetching
  const startUpdates = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }

    // Fetch initial data point
    fetchDataPoint();

    // Set up interval for subsequent updates
    intervalRef.current = setInterval(fetchDataPoint, updateInterval);
    setIsRunning(true);
    setIsPaused(false);
  }, [fetchDataPoint, updateInterval]);

  const stopUpdates = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    setIsRunning(false);
  }, []);

  const pauseUpdates = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    setIsPaused(true);
  }, []);

  const resumeUpdates = useCallback(() => {
    if (!isRunning) return;
    
    intervalRef.current = setInterval(fetchDataPoint, updateInterval);
    setIsPaused(false);
  }, [fetchDataPoint, updateInterval, isRunning]);

  const clearData = useCallback(() => {
    setData([]);
    dataBufferRef.current = [];
    setError(null);
    setLastUpdate(null);
  }, []);

  // Auto-start effect
  useEffect(() => {
    if (autoStart) {
      startUpdates();
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [autoStart, startUpdates]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  const getStatusColor = () => {
    if (error) return 'bg-red-500';
    if (!isRunning) return 'bg-gray-400';
    if (isPaused) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  const getStatusText = () => {
    if (error) return 'Error';
    if (!isRunning) return 'Stopped';
    if (isPaused) return 'Paused';
    return 'Live';
  };

  const ChartComponent = chartType === 'area' ? AreaChart : LineChart;

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center space-x-2">
              <span>{title}</span>
              <div className="flex items-center space-x-2">
                <div className={`w-2 h-2 rounded-full ${getStatusColor()}`} />
                <Badge variant={error ? 'error' : isRunning ? 'success' : 'secondary'}>
                  {getStatusText()}
                </Badge>
              </div>
            </CardTitle>
            {description && <CardDescription>{description}</CardDescription>}
          </div>
          
          {showControls && (
            <div className="flex items-center space-x-2">
              {!isRunning ? (
                <Button size="sm" onClick={startUpdates}>
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1m-6-8h1m4 0h1M9 6h1m4 0h1M9 2h1m4 0h1" />
                  </svg>
                  Start
                </Button>
              ) : isPaused ? (
                <Button size="sm" onClick={resumeUpdates}>
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1m-6-8h1m4 0h1M9 6h1m4 0h1M9 2h1m4 0h1" />
                  </svg>
                  Resume
                </Button>
              ) : (
                <Button size="sm" variant="outline" onClick={pauseUpdates}>
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 9v6m4-6v6" />
                  </svg>
                  Pause
                </Button>
              )}
              
              <Button size="sm" variant="outline" onClick={stopUpdates}>
                <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
                </svg>
                Stop
              </Button>
              
              <Button size="sm" variant="outline" onClick={clearData}>
                <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
                Clear
              </Button>
            </div>
          )}
        </div>
        
        {/* Status Information */}
        <div className="flex items-center space-x-4 text-sm text-muted-foreground">
          <span>Data Points: {data.length}/{maxDataPoints}</span>
          <span>Update Interval: {updateInterval / 1000}s</span>
          {lastUpdate && (
            <span>Last Update: {lastUpdate.toLocaleTimeString()}</span>
          )}
        </div>
        
        {error && (
          <div className="text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-2 rounded">
            Error: {error}
          </div>
        )}
      </CardHeader>
      
      <CardContent>
        {data.length > 0 ? (
          <ChartComponent
            data={data}
            series={series}
            height={height}
            xAxisFormatter={formatters?.xAxis || ((value) => 
              new Date(value).toLocaleTimeString()
            )}
            yAxisFormatter={formatters?.yAxis}
            tooltipFormatter={formatters?.tooltip}
            animate={false} // Disable animation for real-time updates
            syncId="realtime"
          />
        ) : (
          <div className="flex items-center justify-center h-64 text-muted-foreground">
            <div className="text-center">
              <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              <p>No data available</p>
              <p className="text-sm">
                {isRunning ? 'Waiting for data...' : 'Click Start to begin monitoring'}
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};