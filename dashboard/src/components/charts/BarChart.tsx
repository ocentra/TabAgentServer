import React from 'react';
import {
  BarChart as RechartsBarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  TooltipProps,
  Cell,
} from 'recharts';
import { formatNumber } from '@/lib/utils';

export interface BarChartDataPoint {
  name: string;
  [key: string]: string | number;
}

export interface BarChartSeries {
  key: string;
  name: string;
  color: string;
  stackId?: string;
}

export interface BarChartProps {
  data: BarChartDataPoint[];
  series: BarChartSeries[];
  height?: number;
  showGrid?: boolean;
  showLegend?: boolean;
  showTooltip?: boolean;
  layout?: 'horizontal' | 'vertical';
  yAxisFormatter?: (value: any) => string;
  tooltipFormatter?: (value: any, name: string) => [string, string];
  className?: string;
  animate?: boolean;
  colors?: string[];
  maxBarSize?: number;
}

const CustomTooltip: React.FC<TooltipProps<any, any> & { 
  tooltipFormatter?: (value: any, name: string) => [string, string];
}> = ({ 
  active, 
  payload, 
  label, 
  tooltipFormatter 
}) => {
  if (!active || !payload || !payload.length) {
    return null;
  }

  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-3">
      <p className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">
        {label}
      </p>
      <div className="space-y-1">
        {payload.map((entry, index) => (
          <div key={index} className="flex items-center space-x-2">
            <div 
              className="w-3 h-3 rounded-sm"
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

export const BarChart: React.FC<BarChartProps> = ({
  data,
  series,
  height = 300,
  showGrid = true,
  showLegend = true,
  showTooltip = true,
  layout = 'vertical',
  yAxisFormatter,
  tooltipFormatter,
  className = '',
  animate = true,
  colors,
  maxBarSize = 50,
}) => {
  // If colors array is provided and we have a single series, use different colors for each bar
  const useMultipleColors = colors && series.length === 1;

  return (
    <div className={`w-full ${className}`}>
      <ResponsiveContainer width="100%" height={height}>
        <RechartsBarChart
          data={data}
          layout={layout}
          margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
          maxBarSize={maxBarSize}
        >
          {showGrid && (
            <CartesianGrid 
              strokeDasharray="3 3" 
              className="stroke-gray-200 dark:stroke-gray-700"
            />
          )}
          
          <XAxis
            type={layout === 'vertical' ? 'category' : 'number'}
            dataKey={layout === 'vertical' ? 'name' : undefined}
            tickFormatter={layout === 'horizontal' ? (yAxisFormatter || formatNumber) : undefined}
            className="text-xs fill-gray-600 dark:fill-gray-400"
            axisLine={false}
            tickLine={false}
          />
          
          <YAxis
            type={layout === 'vertical' ? 'number' : 'category'}
            dataKey={layout === 'horizontal' ? 'name' : undefined}
            tickFormatter={layout === 'vertical' ? (yAxisFormatter || formatNumber) : undefined}
            className="text-xs fill-gray-600 dark:fill-gray-400"
            axisLine={false}
            tickLine={false}
          />
          
          {showTooltip && (
            <Tooltip
              content={<CustomTooltip tooltipFormatter={tooltipFormatter} />}
            />
          )}
          
          {showLegend && series.length > 1 && (
            <Legend 
              wrapperStyle={{ 
                paddingTop: '20px',
                fontSize: '12px'
              }}
            />
          )}
          
          {series.map((s) => (
            <Bar
              key={s.key}
              dataKey={s.key}
              name={s.name}
              fill={s.color}
              stackId={s.stackId}
              isAnimationActive={animate}
              animationDuration={300}
            >
              {useMultipleColors && colors.map((color, index) => (
                <Cell key={`cell-${index}`} fill={color} />
              ))}
            </Bar>
          ))}
        </RechartsBarChart>
      </ResponsiveContainer>
    </div>
  );
};