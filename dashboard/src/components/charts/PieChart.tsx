import React from 'react';
import {
  PieChart as RechartsPieChart,
  Pie,
  Cell,
  Tooltip,
  Legend,
  ResponsiveContainer,
  TooltipProps,
} from 'recharts';
import { formatNumber, formatBytes } from '@/lib/utils';

export interface PieChartDataPoint {
  name: string;
  value: number;
  color?: string;
}

export interface PieChartProps {
  data: PieChartDataPoint[];
  height?: number;
  showLegend?: boolean;
  showTooltip?: boolean;
  showLabels?: boolean;
  innerRadius?: number;
  outerRadius?: number;
  colors?: string[];
  valueFormatter?: (value: number) => string;
  tooltipFormatter?: (value: number, name: string) => [string, string];
  className?: string;
  animate?: boolean;
  centerLabel?: {
    value: string | number;
    label: string;
  };
}

const RADIAN = Math.PI / 180;

const renderCustomizedLabel = ({
  cx, cy, midAngle, innerRadius, outerRadius, percent, name
}: any) => {
  const radius = innerRadius + (outerRadius - innerRadius) * 0.5;
  const x = cx + radius * Math.cos(-midAngle * RADIAN);
  const y = cy + radius * Math.sin(-midAngle * RADIAN);

  if (percent < 0.05) return null; // Don't show labels for slices smaller than 5%

  return (
    <text 
      x={x} 
      y={y} 
      fill="white" 
      textAnchor={x > cx ? 'start' : 'end'} 
      dominantBaseline="central"
      className="text-xs font-medium"
    >
      {`${(percent * 100).toFixed(0)}%`}
    </text>
  );
};

const CustomTooltip: React.FC<TooltipProps<any, any> & { 
  valueFormatter?: (value: number) => string;
  tooltipFormatter?: (value: number, name: string) => [string, string];
}> = ({ 
  active, 
  payload,
  valueFormatter,
  tooltipFormatter 
}) => {
  if (!active || !payload || !payload.length) {
    return null;
  }

  const data = payload[0];

  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-3">
      <div className="flex items-center space-x-2 mb-2">
        <div 
          className="w-3 h-3 rounded-full"
          style={{ backgroundColor: data.payload.color || data.color }}
        />
        <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
          {data.name}
        </span>
      </div>
      <div className="text-sm text-gray-600 dark:text-gray-400">
        Value: <span className="font-medium text-gray-900 dark:text-gray-100">
          {tooltipFormatter 
            ? tooltipFormatter(data.value, data.name)[0]
            : valueFormatter 
            ? valueFormatter(data.value)
            : formatNumber(data.value)
          }
        </span>
      </div>
      <div className="text-sm text-gray-600 dark:text-gray-400">
        Percentage: <span className="font-medium text-gray-900 dark:text-gray-100">
          {((data.value / data.payload.total) * 100).toFixed(1)}%
        </span>
      </div>
    </div>
  );
};

const CenterLabel: React.FC<{ 
  cx: number; 
  cy: number; 
  value: string | number; 
  label: string;
  valueFormatter?: (value: number) => string;
}> = ({ cx, cy, value, label, valueFormatter }) => {
  return (
    <g>
      <text 
        x={cx} 
        y={cy - 10} 
        textAnchor="middle" 
        dominantBaseline="middle"
        className="text-2xl font-bold fill-gray-900 dark:fill-gray-100"
      >
        {typeof value === 'number' && valueFormatter 
          ? valueFormatter(value)
          : formatNumber(Number(value))
        }
      </text>
      <text 
        x={cx} 
        y={cy + 15} 
        textAnchor="middle" 
        dominantBaseline="middle"
        className="text-sm fill-gray-600 dark:fill-gray-400"
      >
        {label}
      </text>
    </g>
  );
};

export const PieChart: React.FC<PieChartProps> = ({
  data,
  height = 300,
  showLegend = true,
  showTooltip = true,
  showLabels = false,
  innerRadius = 0,
  outerRadius = 80,
  colors = [
    '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
    '#06b6d4', '#84cc16', '#f97316', '#ec4899', '#6366f1'
  ],
  valueFormatter,
  tooltipFormatter,
  className = '',
  animate = true,
  centerLabel,
}) => {
  // Calculate total for percentage calculations
  const total = data.reduce((sum, item) => sum + item.value, 0);
  const dataWithTotal = data.map(item => ({ ...item, total }));

  // Assign colors to data points
  const dataWithColors = dataWithTotal.map((item, index) => ({
    ...item,
    color: item.color || colors[index % colors.length],
  }));

  return (
    <div className={`w-full ${className}`}>
      <ResponsiveContainer width="100%" height={height}>
        <RechartsPieChart>
          <Pie
            data={dataWithColors}
            cx="50%"
            cy="50%"
            labelLine={false}
            label={showLabels ? renderCustomizedLabel : false}
            outerRadius={outerRadius}
            innerRadius={innerRadius}
            fill="#8884d8"
            dataKey="value"
            isAnimationActive={animate}
            animationDuration={300}
          >
            {dataWithColors.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          
          {showTooltip && (
            <Tooltip
              content={
                <CustomTooltip 
                  valueFormatter={valueFormatter}
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
              formatter={(value, entry) => (
                <span style={{ color: entry.color }}>
                  {value}
                </span>
              )}
            />
          )}
          
          {centerLabel && innerRadius > 0 && (
            <text x="50%" y="50%" textAnchor="middle" dominantBaseline="middle">
              <CenterLabel
                cx={0}
                cy={0}
                value={centerLabel.value}
                label={centerLabel.label}
                valueFormatter={valueFormatter}
              />
            </text>
          )}
        </RechartsPieChart>
      </ResponsiveContainer>
    </div>
  );
};