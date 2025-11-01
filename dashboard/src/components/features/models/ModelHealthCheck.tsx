import React, { useState } from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo } from '../../../types/models';

interface HealthCheckResult {
  status: 'healthy' | 'warning' | 'error';
  checks: {
    id: string;
    name: string;
    status: 'pass' | 'warning' | 'fail';
    message: string;
    details?: string;
  }[];
  timestamp: string;
  duration: number;
}

interface ModelHealthCheckProps {
  model: ModelInfo;
  onRunHealthCheck: (modelId: string) => Promise<HealthCheckResult>;
  className?: string;
}

export const ModelHealthCheck: React.FC<ModelHealthCheckProps> = ({
  model,
  onRunHealthCheck,
  className = '',
}) => {
  const [isRunning, setIsRunning] = useState(false);
  const [lastResult, setLastResult] = useState<HealthCheckResult | null>(null);

  const handleRunHealthCheck = async () => {
    setIsRunning(true);
    try {
      const result = await onRunHealthCheck(model.id);
      setLastResult(result);
    } catch (error) {
      console.error('Health check failed:', error);
      setLastResult({
        status: 'error',
        checks: [{
          id: 'general',
          name: 'General Health',
          status: 'fail',
          message: 'Health check failed to run',
          details: error instanceof Error ? error.message : 'Unknown error',
        }],
        timestamp: new Date().toISOString(),
        duration: 0,
      });
    } finally {
      setIsRunning(false);
    }
  };

  const getStatusIcon = (status: 'pass' | 'warning' | 'fail') => {
    switch (status) {
      case 'pass':
        return '✅';
      case 'warning':
        return '⚠️';
      case 'fail':
        return '❌';
    }
  };

  const getStatusColor = (status: 'pass' | 'warning' | 'fail') => {
    switch (status) {
      case 'pass':
        return 'text-green-600 dark:text-green-400';
      case 'warning':
        return 'text-yellow-600 dark:text-yellow-400';
      case 'fail':
        return 'text-red-600 dark:text-red-400';
    }
  };

  const getOverallStatusColor = (status: 'healthy' | 'warning' | 'error') => {
    switch (status) {
      case 'healthy':
        return 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800 text-green-800 dark:text-green-200';
      case 'warning':
        return 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-200';
      case 'error':
        return 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-800 dark:text-red-200';
    }
  };

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              Model Health Check
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
          </div>
          <Button
            onClick={handleRunHealthCheck}
            disabled={isRunning || model.status !== 'loaded'}
            size="sm"
          >
            {isRunning ? (
              <>
                <LoadingSpinner size="sm" className="mr-2" />
                Running...
              </>
            ) : (
              'Run Health Check'
            )}
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        {model.status !== 'loaded' ? (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            <p className="text-lg">Model not loaded</p>
            <p className="text-sm mt-1">Load the model to run health checks</p>
          </div>
        ) : (
          <div className="space-y-4">
            {/* Overall Status */}
            {lastResult && (
              <div className={`p-4 border rounded-lg ${getOverallStatusColor(lastResult.status)}`}>
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <span className="text-lg">
                      {lastResult.status === 'healthy' ? '✅' : 
                       lastResult.status === 'warning' ? '⚠️' : '❌'}
                    </span>
                    <span className="font-medium">
                      Overall Status: {lastResult.status.charAt(0).toUpperCase() + lastResult.status.slice(1)}
                    </span>
                  </div>
                  <div className="text-sm">
                    {new Date(lastResult.timestamp).toLocaleString()}
                  </div>
                </div>
                <div className="text-sm mt-1">
                  Health check completed in {lastResult.duration}ms
                </div>
              </div>
            )}

            {/* Health Check Results */}
            {lastResult ? (
              <div className="space-y-3">
                <h4 className="font-medium text-gray-900 dark:text-white">Check Results</h4>
                {lastResult.checks.map((check) => (
                  <div key={check.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex items-start space-x-3">
                        <span className="text-lg mt-0.5">{getStatusIcon(check.status)}</span>
                        <div className="flex-1">
                          <div className="flex items-center space-x-2">
                            <h5 className="font-medium text-gray-900 dark:text-white">{check.name}</h5>
                            <span className={`text-sm font-medium ${getStatusColor(check.status)}`}>
                              {check.status.toUpperCase()}
                            </span>
                          </div>
                          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {check.message}
                          </p>
                          {check.details && (
                            <details className="mt-2">
                              <summary className="text-xs text-gray-500 dark:text-gray-400 cursor-pointer hover:text-gray-700 dark:hover:text-gray-300">
                                Show details
                              </summary>
                              <pre className="text-xs text-gray-600 dark:text-gray-400 mt-1 bg-gray-50 dark:bg-gray-800 p-2 rounded overflow-x-auto">
                                {check.details}
                              </pre>
                            </details>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                <div className="text-4xl mb-2">🔍</div>
                <p className="text-lg">No health check results</p>
                <p className="text-sm mt-1">Run a health check to validate model functionality</p>
              </div>
            )}

            {/* Health Check Information */}
            <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
              <div className="flex items-start space-x-2">
                <span className="text-blue-600 dark:text-blue-400 mt-0.5">ℹ️</span>
                <div className="text-sm text-blue-800 dark:text-blue-200">
                  <p className="font-medium mb-1">Health Check Includes:</p>
                  <ul className="space-y-1 text-xs">
                    <li>• Model loading and initialization status</li>
                    <li>• Memory usage and allocation validation</li>
                    <li>• Basic inference functionality test</li>
                    <li>• Configuration and parameter validation</li>
                    <li>• GPU/CPU compatibility and performance</li>
                    <li>• Model file integrity verification</li>
                  </ul>
                </div>
              </div>
            </div>

            {/* Quick Actions */}
            <div className="flex space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  // TODO: Implement model restart
                  console.log('Restart model:', model.id);
                }}
                disabled={model.status !== 'loaded'}
              >
                Restart Model
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  // TODO: Implement model repair
                  console.log('Repair model:', model.id);
                }}
                disabled={model.status !== 'loaded' && model.status !== 'error'}
              >
                Repair Model
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  // TODO: Implement model validation
                  console.log('Validate model files:', model.id);
                }}
              >
                Validate Files
              </Button>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};