import React from 'react';
import {
  AreaChart as RechartsAreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  TooltipProps,
} from 'recharts';
import { formatDate, formatNumber } from '@/lib/utils';

export interface AreaChartDataPoint {
  timestamp: string | number;
  [key: string]: string | number;
}

export interface AreaChartSeries {
  key: string;
  name: string;
  color: string;
  fillOpacity?: number;
  strokeWidth?: number;
  stackId?: string;
}

export interface AreaChartProps {
  data: AreaChartDataPoint[];
  series: AreaChartSeries[];
  height?: number;
  showGrid?: boolean;
  showLegend?: boolean;
  showTooltip?: boolean;
  xAxisKey?: string;
  xAxisFormatter?: (value: any) => string;
  yAxisFormatter?: (value: any) => string;
  tooltipFormatter?: (value: any, name: string) => [string, string];
  className?: string;
  animate?: boolean;
  syncId?: string;
  stacked?: boolean;
}

const CustomTooltip: React.FC<TooltipProps<any, any> & { 
  xAxisFormatter?: (value: any) => string;
  tooltipFormatter?: (value: any, name: string) => [string, string];
}> = ({ 
  active, 
  payload, 
  label, 
  xAxisFormatter,
  tooltipFormatter 
}) => {
  if (!active || !payload || !payload.length) {
    return null;
  }

  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-3">
      <p className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">
        {xAxisFormatter ? xAxisFormatter(label) : formatDate(label)}
      </p>
      <div className="space-y-1">
        {payload.map((entry, index) => (
          <div key={index} className="flex items-center space-x-2">
            <div 
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {entry.name}:
            </span>
            <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
              {tooltipFormatter 
                ? tooltipFormatter(entry.value, entry.name)[0]
                : formatNumber(entry.value)
              }
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

export const AreaChart: React.FC<AreaChartProps> = ({
  data,
  series,
  height = 300,
  showGrid = true,
  showLegend = true,
  showTooltip = true,
  xAxisKey = 'timestamp',
  xAxisFormatter,
  yAxisFormatter,
  tooltipFormatter,
  className = '',
  animate = true,
  syncId,
  stacked = false,
}) => {
  return (
    <div className={`w-full ${className}`}>
      <ResponsiveContainer width="100%" height={height}>
        <RechartsAreaChart
          data={data}
          margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
          syncId={syncId}
        >
          <defs>
            {series.map((s) => (
              <linearGradient key={s.key} id={`gradient-${s.key}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={s.color} stopOpacity={s.fillOpacity || 0.8} />
                <stop offset="95%" stopColor={s.color} stopOpacity={0.1} />
              </linearGradient>
            ))}
          </defs>
          
          {showGrid && (
            <CartesianGrid 
              strokeDasharray="3 3" 
              className="stroke-gray-200 dark:stroke-gray-700"
            />
          )}
          
          <XAxis
            dataKey={xAxisKey}
            tickFormatter={xAxisFormatter || ((value) => {
              if (typeof value === 'number') {
                return new Date(value).toLocaleTimeString();
              }
              return formatDate(value).split(' ')[1] || value;
            })}
            className="text-xs fill-gray-600 dark:fill-gray-400"
            axisLine={false}
            tickLine={false}
          />
          
          <YAxis
            tickFormatter={yAxisFormatter || formatNumber}
            className="text-xs fill-gray-600 dark:fill-gray-400"
            axisLine={false}
            tickLine={false}
          />
          
          {showTooltip && (
            <Tooltip
              content={
                <CustomTooltip 
                  xAxisFormatter={xAxisFormatter}
                  tooltipFormatter={tooltipFormatter}
                />
              }
            />
          )}
          
          {showLegend && (
            <Legend 
              wrapperStyle={{ 
                paddingTop: '20px',
                fontSize: '12px'
              }}
            />
          )}
          
          {series.map((s) => (
            <Area
              key={s.key}
              type="monotone"
              dataKey={s.key}
              name={s.name}
              stackId={stacked ? 'stack' : undefined}
              stroke={s.color}
              strokeWidth={s.strokeWidth || 2}
              fill={`url(#gradient-${s.key})`}
              isAnimationActive={animate}
              animationDuration={300}
            />
          ))}
        </RechartsAreaChart>
      </ResponsiveContainer>
    </div>
  );
};