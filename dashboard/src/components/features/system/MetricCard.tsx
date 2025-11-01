import React from 'react';
import { Card } from '../../ui/Card';

interface MetricCardProps {
  title: string;
  value: string | number;
  unit?: string;
  change?: number;
  changeLabel?: string;
  status?: 'good' | 'warning' | 'error' | 'neutral';
  icon?: React.ReactNode;
  className?: string;
  onClick?: () => void;
}

export const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  unit,
  change,
  changeLabel,
  status = 'neutral',
  icon,
  className = '',
  onClick
}) => {
  const getStatusColor = () => {
    switch (status) {
      case 'good':
        return 'text-green-600 dark:text-green-400';
      case 'warning':
        return 'text-yellow-600 dark:text-yellow-400';
      case 'error':
        return 'text-red-600 dark:text-red-400';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  };

  const getChangeColor = () => {
    if (change === undefined) return '';
    if (change > 0) return 'text-green-600 dark:text-green-400';
    if (change < 0) return 'text-red-600 dark:text-red-400';
    return 'text-gray-600 dark:text-gray-400';
  };

  const formatChange = () => {
    if (change === undefined) return null;
    const sign = change > 0 ? '+' : '';
    return `${sign}${change.toFixed(1)}%`;
  };

  return (
    <Card 
      className={`p-4 transition-all duration-200 ${
        onClick ? 'cursor-pointer hover:shadow-md hover:scale-105' : ''
      } ${className}`}
      onClick={onClick}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center space-x-2 mb-1">
            {icon && (
              <div className={`flex-shrink-0 ${getStatusColor()}`}>
                {icon}
              </div>
            )}
            <h3 className="text-sm font-medium text-gray-600 dark:text-gray-400 truncate">
              {title}
            </h3>
          </div>
          
          <div className="flex items-baseline space-x-1">
            <span className="text-2xl font-bold text-gray-900 dark:text-white">
              {typeof value === 'number' ? value.toLocaleString() : value}
            </span>
            {unit && (
              <span className="text-sm text-gray-500 dark:text-gray-400">
                {unit}
              </span>
            )}
          </div>
          
          {(change !== undefined || changeLabel) && (
            <div className="flex items-center space-x-2 mt-2">
              {change !== undefined && (
                <span className={`text-sm font-medium ${getChangeColor()}`}>
                  {formatChange()}
                </span>
              )}
              {changeLabel && (
                <span className="text-xs text-gray-500 dark:text-gray-400">
                  {changeLabel}
                </span>
              )}
            </div>
          )}
        </div>
        
        {status !== 'neutral' && (
          <div className="flex-shrink-0 ml-2">
            <div className={`w-3 h-3 rounded-full ${
              status === 'good' 
                ? 'bg-green-500' 
                : status === 'warning' 
                ? 'bg-yellow-500' 
                : status === 'error'
                ? 'bg-red-500'
                : 'bg-gray-500'
            }`} />
          </div>
        )}
      </div>
    </Card>
  );
};

// Specialized metric cards for common use cases

interface PerformanceMetricCardProps {
  title: string;
  value: number;
  unit: string;
  target?: number;
  className?: string;
}

export const PerformanceMetricCard: React.FC<PerformanceMetricCardProps> = ({
  title,
  value,
  unit,
  target,
  className
}) => {
  const getStatus = (): 'good' | 'warning' | 'error' | 'neutral' => {
    if (target === undefined) return 'neutral';
    
    // For response time, lower is better
    if (unit.includes('ms') || unit.includes('time')) {
      if (value <= target) return 'good';
      if (value <= target * 1.5) return 'warning';
      return 'error';
    }
    
    // For throughput metrics, higher is better
    if (unit.includes('/s') || unit.includes('rate')) {
      if (value >= target) return 'good';
      if (value >= target * 0.8) return 'warning';
      return 'error';
    }
    
    // For percentage metrics
    if (unit.includes('%')) {
      if (value <= 70) return 'good';
      if (value <= 85) return 'warning';
      return 'error';
    }
    
    return 'neutral';
  };

  return (
    <MetricCard
      title={title}
      value={value}
      unit={unit}
      status={getStatus()}
      className={className}
    />
  );
};

interface ResourceMetricCardProps {
  title: string;
  used: number;
  total: number;
  unit: string;
  className?: string;
}

export const ResourceMetricCard: React.FC<ResourceMetricCardProps> = ({
  title,
  used,
  total,
  unit,
  className
}) => {
  const percentage = total > 0 ? (used / total) * 100 : 0;
  
  const getStatus = (): 'good' | 'warning' | 'error' => {
    if (percentage <= 70) return 'good';
    if (percentage <= 85) return 'warning';
    return 'error';
  };

  return (
    <MetricCard
      title={title}
      value={`${used.toFixed(1)} / ${total.toFixed(1)}`}
      unit={unit}
      change={percentage}
      changeLabel="usage"
      status={getStatus()}
      className={className}
    />
  );
};