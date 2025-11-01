// Dashboard-specific types
export interface DashboardConfig {
  theme: 'light' | 'dark' | 'system';
  refreshInterval: number;
  autoRefresh: boolean;
  notifications: boolean;
}

export interface NavigationItem {
  id: string;
  label: string;
  path: string;
  icon: string;
  badge?: number;
}

export interface MetricCardData {
  title: string;
  value: string | number;
  change?: number;
  trend?: 'up' | 'down' | 'stable';
  format?: 'number' | 'percentage' | 'bytes' | 'duration';
}

export interface ChartDataPoint {
  timestamp: string;
  value: number;
  label?: string;
}

export interface NotificationItem {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  title: string;
  message: string;
  timestamp: string;
  read: boolean;
}