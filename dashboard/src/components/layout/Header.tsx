import React from 'react';
import ThemeToggle from '@/components/ui/ThemeToggle';

interface StatusIndicatorProps {
  status: 'online' | 'offline' | 'degraded';
  label: string;
}

const StatusIndicator: React.FC<StatusIndicatorProps> = ({ status }) => {
  const statusConfig = {
    online: { color: 'bg-success-500', text: 'Online' },
    offline: { color: 'bg-error-500', text: 'Offline' },
    degraded: { color: 'bg-warning-500', text: 'Degraded' },
  };

  const config = statusConfig[status];

  return (
    <div className="flex items-center space-x-2">
      <div className={`w-2 h-2 ${config.color} rounded-full animate-pulse`}></div>
      <span className="text-sm text-gray-600 dark:text-gray-300">{config.text}</span>
    </div>
  );
};

interface NotificationBellProps {
  count?: number;
}

const NotificationBell: React.FC<NotificationBellProps> = ({ count = 0 }) => {
  return (
    <button className="relative p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-all duration-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg focus-ring">
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
      </svg>
      {count > 0 && (
        <span className="absolute -top-1 -right-1 bg-error-500 text-white text-xs rounded-full h-5 w-5 flex items-center justify-center animate-bounce-subtle">
          {count > 99 ? '99+' : count}
        </span>
      )}
    </button>
  );
};

const UserMenu: React.FC = () => {
  return (
    <div className="relative">
      <button className="w-8 h-8 bg-primary-500 hover:bg-primary-600 rounded-full flex items-center justify-center transition-all duration-200 focus-ring shadow-sm">
        <span className="text-white text-sm font-medium">A</span>
      </button>
    </div>
  );
};

const Header: React.FC = () => {
  // TODO: These will be connected to actual API in later tasks
  const systemStatus = 'online'; // This will come from API
  const notificationCount = 3; // This will come from notifications store

  return (
    <header className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50 backdrop-blur-sm bg-white/95 dark:bg-gray-800/95">
      <div className="px-6 py-4 flex justify-between items-center">
        <div className="flex items-center space-x-6">
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-gradient-to-br from-primary-500 to-primary-600 rounded-lg flex items-center justify-center shadow-sm">
              <svg className="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
                <path d="M13 3L4 14h7v7l9-11h-7V3z" />
              </svg>
            </div>
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">
              TabAgent Dashboard
            </h1>
          </div>
          
          <div className="hidden md:flex items-center space-x-4">
            <StatusIndicator status={systemStatus as any} label="System Status" />
            <div className="h-4 w-px bg-gray-300 dark:bg-gray-600"></div>
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-500 dark:text-gray-400">Server:</span>
              <span className="text-sm font-medium text-gray-900 dark:text-white">localhost:3000</span>
            </div>
          </div>
        </div>
        
        <div className="flex items-center space-x-2">
          <NotificationBell count={notificationCount} />
          <ThemeToggle />
          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600 mx-2"></div>
          <UserMenu />
        </div>
      </div>
    </header>
  );
};

export default Header;