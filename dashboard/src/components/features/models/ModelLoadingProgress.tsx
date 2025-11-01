import React from 'react';
import { LoadingSpinner } from '../../ui/LoadingSpinner';

interface ModelLoadingProgressProps {
  modelName: string;
  progress?: number; // 0-100
  stage?: string;
  className?: string;
}

export const ModelLoadingProgress: React.FC<ModelLoadingProgressProps> = ({
  modelName,
  progress,
  stage,
  className = '',
}) => {
  return (
    <div className={`bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 ${className}`}>
      <div className="flex items-center space-x-3">
        <LoadingSpinner size="sm" />
        <div className="flex-1">
          <div className="flex items-center justify-between mb-1">
            <span className="text-sm font-medium text-blue-900 dark:text-blue-100">
              Loading {modelName}
            </span>
            {progress !== undefined && (
              <span className="text-xs text-blue-700 dark:text-blue-300">
                {progress.toFixed(0)}%
              </span>
            )}
          </div>
          
          {progress !== undefined && (
            <div className="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-2 mb-2">
              <div
                className="bg-blue-600 dark:bg-blue-400 h-2 rounded-full transition-all duration-300"
                style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
              />
            </div>
          )}
          
          {stage && (
            <p className="text-xs text-blue-700 dark:text-blue-300">
              {stage}
            </p>
          )}
        </div>
      </div>
    </div>
  );
};