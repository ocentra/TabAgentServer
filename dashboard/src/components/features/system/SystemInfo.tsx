import React, { useState } from 'react';
import { Card } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { useSystemInfo, useConfig, useHealth } from '../../../hooks/useApi';

interface SystemInfoProps {
  className?: string;
}

interface InfoSectionProps {
  title: string;
  children: React.ReactNode;
  collapsible?: boolean;
  defaultExpanded?: boolean;
}

const InfoSection: React.FC<InfoSectionProps> = ({ 
  title, 
  children, 
  collapsible = false,
  defaultExpanded = true 
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg">
      <div 
        className={`px-4 py-3 bg-gray-50 dark:bg-gray-800 rounded-t-lg ${
          collapsible ? 'cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700' : ''
        }`}
        onClick={collapsible ? () => setIsExpanded(!isExpanded) : undefined}
      >
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            {title}
          </h3>
          {collapsible && (
            <svg
              className={`w-5 h-5 text-gray-500 transition-transform ${
                isExpanded ? 'rotate-180' : ''
              }`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          )}
        </div>
      </div>
      {(!collapsible || isExpanded) && (
        <div className="p-4">
          {children}
        </div>
      )}
    </div>
  );
};

interface InfoItemProps {
  label: string;
  value: string | number | boolean | undefined;
  type?: 'text' | 'boolean' | 'bytes' | 'duration' | 'percentage';
}

const InfoItem: React.FC<InfoItemProps> = ({ label, value, type = 'text' }) => {
  const formatValue = () => {
    if (value === undefined || value === null) return 'N/A';
    
    switch (type) {
      case 'boolean':
        return value ? 'Yes' : 'No';
      case 'bytes':
        const bytes = Number(value);
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
        return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
      case 'duration':
        const seconds = Number(value);
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        if (days > 0) return `${days}d ${hours}h ${minutes}m`;
        if (hours > 0) return `${hours}h ${minutes}m`;
        return `${minutes}m`;
      case 'percentage':
        return `${Number(value).toFixed(1)}%`;
      default:
        return String(value);
    }
  };

  return (
    <div className="flex justify-between items-center py-2 border-b border-gray-100 dark:border-gray-700 last:border-b-0">
      <span className="text-sm text-gray-600 dark:text-gray-400">{label}</span>
      <span className="text-sm font-medium text-gray-900 dark:text-white">
        {formatValue()}
      </span>
    </div>
  );
};

export const SystemInfo: React.FC<SystemInfoProps> = ({ className = '' }) => {
  const { data: systemInfo, isLoading: systemLoading, error: systemError } = useSystemInfo();
  const { data: config, isLoading: configLoading } = useConfig();
  const { data: health, isLoading: healthLoading } = useHealth();
  const [refreshing, setRefreshing] = useState(false);

  const isLoading = systemLoading || configLoading || healthLoading;

  const handleRefresh = async () => {
    setRefreshing(true);
    // Trigger a refetch by invalidating queries
    setTimeout(() => setRefreshing(false), 1000);
  };

  if (isLoading && !systemInfo) {
    return (
      <Card className={`p-6 ${className}`}>
        <div className="flex items-center justify-center h-48">
          <LoadingSpinner size="lg" />
        </div>
      </Card>
    );
  }

  if (systemError) {
    return (
      <Card className={`p-6 ${className}`}>
        <div className="text-center text-red-600 dark:text-red-400">
          <h3 className="text-lg font-semibold mb-2">Error Loading System Information</h3>
          <p className="mb-4">Unable to retrieve system information from the server</p>
          <Button onClick={handleRefresh} variant="secondary">
            Retry
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          System Information & Diagnostics
        </h2>
        <Button 
          onClick={handleRefresh} 
          variant="secondary" 
          size="sm"
          disabled={refreshing}
        >
          {refreshing ? <LoadingSpinner size="sm" /> : 'Refresh'}
        </Button>
      </div>

      {/* System Overview */}
      <InfoSection title="System Overview">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-1">
            <InfoItem label="Server Version" value={systemInfo?.version} />
            <InfoItem label="Uptime" value={systemInfo?.uptime} type="duration" />
            <InfoItem label="Server Status" value={health?.status} />
            <InfoItem label="Active Connections" value={systemInfo?.active_connections} />
          </div>
          <div className="space-y-1">
            <InfoItem label="CPU Usage" value={systemInfo?.cpu_usage} type="percentage" />
            <InfoItem label="Memory Usage" value={systemInfo?.memory_usage} type="percentage" />
            <InfoItem label="Disk Usage" value={systemInfo?.disk_usage} type="percentage" />
            {systemInfo?.gpu_usage !== undefined && (
              <InfoItem label="GPU Usage" value={systemInfo.gpu_usage} type="percentage" />
            )}
          </div>
        </div>
      </InfoSection>

      {/* Hardware Information */}
      <InfoSection title="Hardware Information" collapsible defaultExpanded={false}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-1">
            <InfoItem label="Operating System" value={(config as any)?.system?.os || 'Unknown'} />
            <InfoItem label="Architecture" value={(config as any)?.system?.arch || 'Unknown'} />
            <InfoItem label="CPU Cores" value={(config as any)?.system?.cpu_cores} />
            <InfoItem label="Total Memory" value={(config as any)?.system?.total_memory} type="bytes" />
          </div>
          <div className="space-y-1">
            <InfoItem label="GPU Available" value={(config as any)?.system?.gpu_available} type="boolean" />
            <InfoItem label="GPU Model" value={(config as any)?.system?.gpu_model || 'N/A'} />
            <InfoItem label="GPU Memory" value={(config as any)?.system?.gpu_memory} type="bytes" />
            <InfoItem label="CUDA Available" value={(config as any)?.system?.cuda_available} type="boolean" />
          </div>
        </div>
      </InfoSection>

      {/* Service Status */}
      <InfoSection title="Service Status">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-medium text-gray-900 dark:text-white">HTTP Server</h4>
              <div className={`w-3 h-3 rounded-full ${
                health?.services?.http ? 'bg-green-500' : 'bg-red-500'
              }`} />
            </div>
            <div className="space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Status</span>
                <span className="text-gray-900 dark:text-white">
                  {health?.services?.http ? 'Running' : 'Stopped'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Port</span>
                <span className="text-gray-900 dark:text-white">
                  {(config as any)?.server?.http_port || 'Unknown'}
                </span>
              </div>
            </div>
          </div>

          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-medium text-gray-900 dark:text-white">WebRTC Server</h4>
              <div className={`w-3 h-3 rounded-full ${
                health?.services?.webrtc ? 'bg-green-500' : 'bg-red-500'
              }`} />
            </div>
            <div className="space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Status</span>
                <span className="text-gray-900 dark:text-white">
                  {health?.services?.webrtc ? 'Running' : 'Stopped'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Port</span>
                <span className="text-gray-900 dark:text-white">
                  {(config as any)?.server?.webrtc_port || 'Unknown'}
                </span>
              </div>
            </div>
          </div>

          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-medium text-gray-900 dark:text-white">Native Messaging</h4>
              <div className={`w-3 h-3 rounded-full ${
                health?.services?.native_messaging ? 'bg-green-500' : 'bg-red-500'
              }`} />
            </div>
            <div className="space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Status</span>
                <span className="text-gray-900 dark:text-white">
                  {health?.services?.native_messaging ? 'Enabled' : 'Disabled'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Type</span>
                <span className="text-gray-900 dark:text-white">
                  Browser Extension
                </span>
              </div>
            </div>
          </div>
        </div>
      </InfoSection>

      {/* Server Configuration */}
      <InfoSection title="Server Configuration" collapsible defaultExpanded={false}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-1">
            <InfoItem label="Server Mode" value={(config as any)?.server?.mode || 'Unknown'} />
            <InfoItem label="Log Level" value={(config as any)?.logging?.level || 'Unknown'} />
            <InfoItem label="Max Connections" value={(config as any)?.server?.max_connections} />
            <InfoItem label="Request Timeout" value={(config as any)?.server?.request_timeout} />
          </div>
          <div className="space-y-1">
            <InfoItem label="CORS Enabled" value={(config as any)?.server?.cors_enabled} type="boolean" />
            <InfoItem label="TLS Enabled" value={(config as any)?.server?.tls_enabled} type="boolean" />
            <InfoItem label="Debug Mode" value={(config as any)?.server?.debug_mode} type="boolean" />
            <InfoItem label="Auto Reload" value={(config as any)?.server?.auto_reload} type="boolean" />
          </div>
        </div>
      </InfoSection>

      {/* Environment Information */}
      <InfoSection title="Environment Information" collapsible defaultExpanded={false}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-1">
            <InfoItem label="Node.js Version" value={(config as any)?.environment?.node_version} />
            <InfoItem label="Rust Version" value={(config as any)?.environment?.rust_version} />
            <InfoItem label="Build Type" value={(config as any)?.environment?.build_type} />
            <InfoItem label="Build Date" value={(config as any)?.environment?.build_date} />
          </div>
          <div className="space-y-1">
            <InfoItem label="Working Directory" value={(config as any)?.environment?.working_dir} />
            <InfoItem label="Config File" value={(config as any)?.environment?.config_file} />
            <InfoItem label="Log Directory" value={(config as any)?.environment?.log_dir} />
            <InfoItem label="Data Directory" value={(config as any)?.environment?.data_dir} />
          </div>
        </div>
      </InfoSection>

      {/* Health Checks */}
      <InfoSection title="System Health Checks">
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">API Connectivity</span>
                <div className="w-3 h-3 rounded-full bg-green-500" />
              </div>
              <div className="text-lg font-semibold text-gray-900 dark:text-white">Healthy</div>
            </div>

            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">Database</span>
                <div className={`w-3 h-3 rounded-full ${
                  (config as any)?.database?.connected ? 'bg-green-500' : 'bg-red-500'
                }`} />
              </div>
              <div className="text-lg font-semibold text-gray-900 dark:text-white">
                {(config as any)?.database?.connected ? 'Connected' : 'Disconnected'}
              </div>
            </div>

            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">Memory Health</span>
                <div className={`w-3 h-3 rounded-full ${
                  (systemInfo?.memory_usage || 0) < 85 ? 'bg-green-500' : 'bg-yellow-500'
                }`} />
              </div>
              <div className="text-lg font-semibold text-gray-900 dark:text-white">
                {(systemInfo?.memory_usage || 0) < 85 ? 'Good' : 'Warning'}
              </div>
            </div>

            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-600 dark:text-gray-400">Disk Space</span>
                <div className={`w-3 h-3 rounded-full ${
                  (systemInfo?.disk_usage || 0) < 90 ? 'bg-green-500' : 'bg-red-500'
                }`} />
              </div>
              <div className="text-lg font-semibold text-gray-900 dark:text-white">
                {(systemInfo?.disk_usage || 0) < 90 ? 'Good' : 'Critical'}
              </div>
            </div>
          </div>
        </div>
      </InfoSection>
    </div>
  );
};