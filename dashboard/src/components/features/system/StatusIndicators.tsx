import React from 'react';

interface ServiceStatus {
  http: boolean;
  webrtc: boolean;
  native_messaging: boolean;
}

interface StatusIndicatorsProps {
  services?: ServiceStatus;
  className?: string;
}

interface StatusIndicatorProps {
  label: string;
  status: boolean;
  description?: string;
}

const StatusIndicator: React.FC<StatusIndicatorProps> = ({ 
  label, 
  status, 
  description 
}) => {
  return (
    <div className="flex items-center justify-between p-4 bg-white dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600">
      <div className="flex items-center space-x-3">
        <div className={`w-4 h-4 rounded-full ${
          status ? 'bg-green-500' : 'bg-red-500'
        }`} />
        <div>
          <div className="font-medium text-gray-900 dark:text-white">
            {label}
          </div>
          {description && (
            <div className="text-sm text-gray-600 dark:text-gray-400">
              {description}
            </div>
          )}
        </div>
      </div>
      <div className={`px-3 py-1 rounded-full text-sm font-medium ${
        status 
          ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
          : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
      }`}>
        {status ? 'Online' : 'Offline'}
      </div>
    </div>
  );
};

export const StatusIndicators: React.FC<StatusIndicatorsProps> = ({ 
  services, 
  className = '' 
}) => {
  if (!services) {
    return (
      <div className={`space-y-3 ${className}`}>
        <div className="text-center text-gray-500 dark:text-gray-400 py-8">
          Service status information unavailable
        </div>
      </div>
    );
  }

  const indicators = [
    {
      key: 'http',
      label: 'HTTP Server',
      status: services.http,
      description: 'REST API and web interface'
    },
    {
      key: 'webrtc',
      label: 'WebRTC Server',
      status: services.webrtc,
      description: 'Real-time communication'
    },
    {
      key: 'native_messaging',
      label: 'Native Messaging',
      status: services.native_messaging,
      description: 'Browser extension communication'
    }
  ];

  return (
    <div className={`space-y-3 ${className}`}>
      {indicators.map((indicator) => (
        <StatusIndicator
          key={indicator.key}
          label={indicator.label}
          status={indicator.status}
          description={indicator.description}
        />
      ))}
    </div>
  );
};