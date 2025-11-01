import React from 'react';
import { Badge } from '../../ui/Badge';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo } from '../../../types/models';

interface ModelStatusIndicatorProps {
  status: ModelInfo['status'];
  size?: 'sm' | 'md' | 'lg';
  showIcon?: boolean;
}

export const ModelStatusIndicator: React.FC<ModelStatusIndicatorProps> = ({
  status,
  size = 'md',
  showIcon = true,
}) => {
  const getStatusConfig = (status: ModelInfo['status']) => {
    switch (status) {
      case 'loaded':
        return {
          variant: 'success' as const,
          icon: '✓',
          text: 'Loaded',
          pulse: false,
        };
      case 'loading':
        return {
          variant: 'warning' as const,
          icon: <LoadingSpinner size="sm" />,
          text: 'Loading',
          pulse: true,
        };
      case 'unloaded':
        return {
          variant: 'default' as const,
          icon: '○',
          text: 'Unloaded',
          pulse: false,
        };
      case 'error':
        return {
          variant: 'error' as const,
          icon: '⚠️',
          text: 'Error',
          pulse: false,
        };
      default:
        return {
          variant: 'default' as const,
          icon: '?',
          text: 'Unknown',
          pulse: false,
        };
    }
  };

  const config = getStatusConfig(status);
  
  const sizeClasses = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-sm px-2.5 py-1',
    lg: 'text-base px-3 py-1.5',
  };

  return (
    <Badge 
      variant={config.variant}
      className={`
        ${sizeClasses[size]}
        ${config.pulse ? 'animate-pulse' : ''}
        flex items-center space-x-1
      `}
    >
      {showIcon && (
        <span className="flex items-center">
          {config.icon}
        </span>
      )}
      <span>{config.text}</span>
    </Badge>
  );
};