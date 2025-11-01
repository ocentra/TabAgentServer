import React, { useState, useEffect } from 'react';
import { Button } from '../../ui/Button';
import { Card } from '../../ui/Card';
import { useSystemStats, usePerformanceStats } from '../../../hooks/useApi';

export interface Notification {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  title: string;
  message: string;
  timestamp: Date;
  read: boolean;
  persistent?: boolean;
}

interface NotificationBellProps {
  className?: string;
}

export const NotificationBell: React.FC<NotificationBellProps> = ({ className = '' }) => {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const { data: systemStats } = useSystemStats();
  const { data: performanceStats } = usePerformanceStats();

  // Generate notifications based on system metrics
  useEffect(() => {
    const newNotifications: Notification[] = [];

    if (systemStats) {
      // High CPU usage alert
      if (systemStats.cpu_usage > 90) {
        newNotifications.push({
          id: 'high-cpu',
          type: 'error',
          title: 'High CPU Usage',
          message: `CPU usage is at ${systemStats.cpu_usage.toFixed(1)}%. Consider reducing load or scaling resources.`,
          timestamp: new Date(),
          read: false,
          persistent: true,
        });
      } else if (systemStats.cpu_usage > 80) {
        newNotifications.push({
          id: 'elevated-cpu',
          type: 'warning',
          title: 'Elevated CPU Usage',
          message: `CPU usage is at ${systemStats.cpu_usage.toFixed(1)}%. Monitor for performance impact.`,
          timestamp: new Date(),
          read: false,
        });
      }

      // High memory usage alert
      if (systemStats.memory_usage > 95) {
        newNotifications.push({
          id: 'high-memory',
          type: 'error',
          title: 'Critical Memory Usage',
          message: `Memory usage is at ${systemStats.memory_usage.toFixed(1)}%. System may become unstable.`,
          timestamp: new Date(),
          read: false,
          persistent: true,
        });
      } else if (systemStats.memory_usage > 85) {
        newNotifications.push({
          id: 'elevated-memory',
          type: 'warning',
          title: 'High Memory Usage',
          message: `Memory usage is at ${systemStats.memory_usage.toFixed(1)}%. Consider freeing up memory.`,
          timestamp: new Date(),
          read: false,
        });
      }

      // Low disk space alert
      if (systemStats.disk_usage > 95) {
        newNotifications.push({
          id: 'low-disk',
          type: 'error',
          title: 'Critical Disk Space',
          message: `Disk usage is at ${systemStats.disk_usage.toFixed(1)}%. Free up space immediately.`,
          timestamp: new Date(),
          read: false,
          persistent: true,
        });
      } else if (systemStats.disk_usage > 85) {
        newNotifications.push({
          id: 'disk-warning',
          type: 'warning',
          title: 'Low Disk Space',
          message: `Disk usage is at ${systemStats.disk_usage.toFixed(1)}%. Consider cleaning up files.`,
          timestamp: new Date(),
          read: false,
        });
      }
    }

    if (performanceStats) {
      // High error rate alert
      if (performanceStats.error_rate > 10) {
        newNotifications.push({
          id: 'high-error-rate',
          type: 'error',
          title: 'High Error Rate',
          message: `Error rate is at ${performanceStats.error_rate.toFixed(2)}%. Check system logs for issues.`,
          timestamp: new Date(),
          read: false,
          persistent: true,
        });
      } else if (performanceStats.error_rate > 5) {
        newNotifications.push({
          id: 'elevated-error-rate',
          type: 'warning',
          title: 'Elevated Error Rate',
          message: `Error rate is at ${performanceStats.error_rate.toFixed(2)}%. Monitor for patterns.`,
          timestamp: new Date(),
          read: false,
        });
      }

      // Slow response time alert
      if (performanceStats.average_response_time > 2000) {
        newNotifications.push({
          id: 'slow-response',
          type: 'warning',
          title: 'Slow Response Times',
          message: `Average response time is ${performanceStats.average_response_time.toFixed(0)}ms. Performance may be degraded.`,
          timestamp: new Date(),
          read: false,
        });
      }
    }

    // Update notifications, avoiding duplicates
    setNotifications(prev => {
      const existingIds = new Set(prev.map(n => n.id));
      const filtered = newNotifications.filter(n => !existingIds.has(n.id));
      
      // Remove non-persistent notifications that are no longer relevant
      const updated = prev.filter(n => {
        if (n.persistent) return true;
        return newNotifications.some(newN => newN.id === n.id);
      });
      
      return [...updated, ...filtered];
    });
  }, [systemStats, performanceStats]);

  const unreadCount = notifications.filter(n => !n.read).length;

  const markAsRead = (id: string) => {
    setNotifications(prev =>
      prev.map(n => n.id === id ? { ...n, read: true } : n)
    );
  };

  const markAllAsRead = () => {
    setNotifications(prev => prev.map(n => ({ ...n, read: true })));
  };

  const dismissNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  };

  const clearAll = () => {
    setNotifications([]);
  };

  const getNotificationIcon = (type: Notification['type']) => {
    switch (type) {
      case 'error':
        return (
          <svg className="w-5 h-5 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
      case 'warning':
        return (
          <svg className="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        );
      case 'success':
        return (
          <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
      default:
        return (
          <svg className="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
    }
  };

  return (
    <div className={`relative ${className}`}>
      {/* Notification Bell Button */}
      <Button
        variant="secondary"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
        className="relative"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
        </svg>
        
        {unreadCount > 0 && (
          <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full h-5 w-5 flex items-center justify-center">
            {unreadCount > 9 ? '9+' : unreadCount}
          </span>
        )}
      </Button>

      {/* Notification Dropdown */}
      {isOpen && (
        <div className="absolute right-0 mt-2 w-96 z-50">
          <Card className="shadow-lg border border-gray-200 dark:border-gray-700">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Notifications
                </h3>
                <div className="flex space-x-2">
                  {unreadCount > 0 && (
                    <Button variant="secondary" size="sm" onClick={markAllAsRead}>
                      Mark All Read
                    </Button>
                  )}
                  <Button variant="secondary" size="sm" onClick={clearAll}>
                    Clear All
                  </Button>
                </div>
              </div>
            </div>

            <div className="max-h-96 overflow-y-auto">
              {notifications.length === 0 ? (
                <div className="p-6 text-center text-gray-500 dark:text-gray-400">
                  <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                  </svg>
                  <p>No notifications</p>
                </div>
              ) : (
                <div className="divide-y divide-gray-200 dark:divide-gray-700">
                  {notifications.map((notification) => (
                    <div
                      key={notification.id}
                      className={`p-4 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors ${
                        !notification.read ? 'bg-blue-50 dark:bg-blue-900/20' : ''
                      }`}
                    >
                      <div className="flex items-start space-x-3">
                        <div className="flex-shrink-0 mt-0.5">
                          {getNotificationIcon(notification.type)}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <p className="text-sm font-medium text-gray-900 dark:text-white">
                                {notification.title}
                              </p>
                              <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                                {notification.message}
                              </p>
                              <p className="text-xs text-gray-500 dark:text-gray-500 mt-2">
                                {notification.timestamp.toLocaleTimeString()}
                              </p>
                            </div>
                            <div className="flex space-x-1 ml-2">
                              {!notification.read && (
                                <button
                                  onClick={() => markAsRead(notification.id)}
                                  className="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200"
                                  title="Mark as read"
                                >
                                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                  </svg>
                                </button>
                              )}
                              <button
                                onClick={() => dismissNotification(notification.id)}
                                className="text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300"
                                title="Dismiss"
                              >
                                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {notifications.length > 0 && (
              <div className="p-3 border-t border-gray-200 dark:border-gray-700">
                <Button
                  variant="secondary"
                  size="sm"
                  className="w-full"
                  onClick={() => setIsOpen(false)}
                >
                  View All Notifications
                </Button>
              </div>
            )}
          </Card>
        </div>
      )}

      {/* Click outside to close */}
      {isOpen && (
        <div
          className="fixed inset-0 z-40"
          onClick={() => setIsOpen(false)}
        />
      )}
    </div>
  );
};