import React from 'react';
import { Card } from '../../ui/Card';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { useHealth, useSystemInfo, useServerStatus } from '../../../hooks/useApi';
import { StatusIndicators } from './StatusIndicators';

interface SystemMonitorProps {
  className?: string;
}

export const SystemMonitor: React.FC<SystemMonitorProps> = ({ className = '' }) => {
  const { data: health, isLoading: healthLoading, error: healthError } = useHealth();
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();
  const { data: serverStatus, isLoading: statusLoading } = useServerStatus();

  const isLoading = healthLoading || systemLoading || statusLoading;

  if (isLoading) {
    return (
      <Card className={`p-6 ${className}`}>
        <div className="flex items-center justify-center h-48">
          <LoadingSpinner size="lg" />
        </div>
      </Card>
    );
  }

  if (healthError) {
    return (
      <Card className={`p-6 ${className}`}>
        <div className="text-center text-red-600 dark:text-red-400">
          <h3 className="text-lg font-semibold mb-2">Connection Error</h3>
          <p>Unable to connect to TabAgent server</p>
        </div>
      </Card>
    );
  }

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) {
      return `${days}d ${hours}h ${minutes}m`;
    } else if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  };

  return (
    <Card className={`p-6 ${className}`}>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            System Status
          </h2>
          <div className="flex items-center space-x-2">
            <div className={`w-3 h-3 rounded-full ${
              health?.status === 'healthy' 
                ? 'bg-green-500' 
                : health?.status === 'degraded' 
                ? 'bg-yellow-500' 
                : 'bg-red-500'
            }`} />
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300 capitalize">
              {health?.status || 'Unknown'}
            </span>
          </div>
        </div>

        {/* Server Information */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">Version</div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {systemInfo?.version || 'Unknown'}
            </div>
          </div>
          
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">Uptime</div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {systemInfo?.uptime ? formatUptime(systemInfo.uptime) : 'Unknown'}
            </div>
          </div>
          
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">Active Connections</div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {systemInfo?.active_connections ?? 0}
            </div>
          </div>
          
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">Server Mode</div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {serverStatus?.mode || 'Unknown'}
            </div>
          </div>
        </div>

        {/* Transport Status */}
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Transport Status
          </h3>
          <StatusIndicators services={health?.services} />
        </div>

        {/* Server Configuration */}
        {serverStatus && (
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
              Server Configuration
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {serverStatus.http_port && (
                <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                  <div className="text-sm text-gray-600 dark:text-gray-400">HTTP Port</div>
                  <div className="text-lg font-semibold text-gray-900 dark:text-white">
                    {serverStatus.http_port}
                  </div>
                </div>
              )}
              
              {serverStatus.webrtc_port && (
                <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                  <div className="text-sm text-gray-600 dark:text-gray-400">WebRTC Port</div>
                  <div className="text-lg font-semibold text-gray-900 dark:text-white">
                    {serverStatus.webrtc_port}
                  </div>
                </div>
              )}
              
              {serverStatus.native_messaging_enabled !== undefined && (
                <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                  <div className="text-sm text-gray-600 dark:text-gray-400">Native Messaging</div>
                  <div className="text-lg font-semibold text-gray-900 dark:text-white">
                    {serverStatus.native_messaging_enabled ? 'Enabled' : 'Disabled'}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Last Updated */}
        <div className="text-xs text-gray-500 dark:text-gray-400 text-right">
          Last updated: {new Date().toLocaleTimeString()}
        </div>
      </div>
    </Card>
  );
};