# Modern Dashboard Design Document

## Overview

This design document outlines the architecture and implementation approach for building a modern React + TypeScript dashboard for TabAgent server. The design focuses on creating a professional, real-time interface that provides comprehensive system monitoring, model management, and data exploration capabilities.

**Design Philosophy:**
- **Modern Stack**: React 18 + TypeScript + Vite for optimal developer experience
- **Real-Time First**: WebSocket and polling-based live updates throughout the interface
- **Component-Driven**: Reusable, well-tested components with clear interfaces
- **Type-Safe**: Full TypeScript coverage for API integration and component props
- **Performance-Focused**: Optimized builds, code splitting, and efficient data fetching

## Architecture

### Frontend Technology Stack

```
Modern Dashboard Stack
├── React 18 (UI framework)
├── TypeScript (type safety)
├── Vite (build system)
├── Tailwind CSS (styling)
├── Headless UI (accessible components)
├── Framer Motion (animations)
├── React Query (data fetching)
├── Recharts (data visualization)
├── React Hook Form (form handling)
└── Zustand (state management)
```

### Project Structure

```
dashboard/
├── public/
│   ├── favicon.ico
│   └── index.html
├── src/
│   ├── components/
│   │   ├── ui/                    # Base UI components
│   │   │   ├── Button.tsx
│   │   │   ├── Card.tsx
│   │   │   ├── Input.tsx
│   │   │   └── Modal.tsx
│   │   ├── layout/                # Layout components
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── Layout.tsx
│   │   ├── features/              # Feature-specific components
│   │   │   ├── logs/
│   │   │   │   ├── LogsViewer.tsx
│   │   │   │   ├── LogFilters.tsx
│   │   │   │   └── LogEntry.tsx
│   │   │   ├── models/
│   │   │   │   ├── ModelManager.tsx
│   │   │   │   ├── ModelCard.tsx
│   │   │   │   └── ModelMetrics.tsx
│   │   │   ├── system/
│   │   │   │   ├── SystemMonitor.tsx
│   │   │   │   ├── ResourceCharts.tsx
│   │   │   │   └── StatusIndicators.tsx
│   │   │   └── database/
│   │   │       ├── DatabaseExplorer.tsx
│   │   │       ├── NodeViewer.tsx
│   │   │       └── SearchInterface.tsx
│   │   └── charts/                # Chart components
│   │       ├── LineChart.tsx
│   │       ├── BarChart.tsx
│   │       └── MetricCard.tsx
│   ├── hooks/                     # Custom React hooks
│   │   ├── useApi.ts
│   │   ├── useWebSocket.ts
│   │   ├── useRealTimeData.ts
│   │   └── useLocalStorage.ts
│   ├── lib/                       # Utilities and clients
│   │   ├── api-client.ts
│   │   ├── websocket-client.ts
│   │   ├── utils.ts
│   │   └── constants.ts
│   ├── types/                     # TypeScript type definitions
│   │   ├── api.ts
│   │   ├── dashboard.ts
│   │   ├── logs.ts
│   │   ├── models.ts
│   │   └── system.ts
│   ├── stores/                    # State management
│   │   ├── dashboard-store.ts
│   │   ├── theme-store.ts
│   │   └── settings-store.ts
│   ├── pages/                     # Page components
│   │   ├── Dashboard.tsx
│   │   ├── Logs.tsx
│   │   ├── Models.tsx
│   │   ├── Database.tsx
│   │   └── Settings.tsx
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
└── README.md
```

## Components and Interfaces

### Core API Client

```typescript
// lib/api-client.ts
export class TabAgentApiClient {
  private baseUrl: string;
  private wsUrl: string;

  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
    this.wsUrl = baseUrl.replace('http', 'ws');
  }

  // HTTP API methods
  async getHealth(): Promise<HealthStatus> {
    const response = await fetch(`${this.baseUrl}/v1/health`);
    return response.json();
  }

  async getSystemInfo(): Promise<SystemInfo> {
    const response = await fetch(`${this.baseUrl}/v1/system/info`);
    return response.json();
  }

  async getStats(): Promise<PerformanceStats> {
    const response = await fetch(`${this.baseUrl}/v1/stats`);
    return response.json();
  }

  async listModels(): Promise<ModelInfo[]> {
    const response = await fetch(`${this.baseUrl}/v1/models`);
    return response.json();
  }

  async loadModel(modelId: string): Promise<void> {
    await fetch(`${this.baseUrl}/v1/models/load`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ model_id: modelId }),
    });
  }

  // WebSocket connection for real-time updates
  createWebSocket(path: string): WebSocket {
    return new WebSocket(`${this.wsUrl}${path}`);
  }
}
```

### Real-Time Data Hooks

```typescript
// hooks/useRealTimeData.ts
export const useSystemStats = () => {
  return useQuery({
    queryKey: ['system-stats'],
    queryFn: () => apiClient.getStats(),
    refetchInterval: 5000, // Update every 5 seconds
    staleTime: 2000,
  });
};

export const useLogStream = (filters: LogFilters) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  
  useWebSocket('/ws/logs', {
    onMessage: (event) => {
      const newLog: LogEntry = JSON.parse(event.data);
      if (matchesFilters(newLog, filters)) {
        setLogs(prev => [newLog, ...prev].slice(0, 1000)); // Keep last 1000
      }
    },
  });

  return { logs, setLogs };
};

export const useModelMetrics = () => {
  return useQuery({
    queryKey: ['model-metrics'],
    queryFn: () => apiClient.getModelMetrics(),
    refetchInterval: 3000,
  });
};
```

### Dashboard Layout

```typescript
// components/layout/Layout.tsx
export const Layout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { theme } = useThemeStore();
  
  return (
    <div className={`min-h-screen ${theme === 'dark' ? 'dark' : ''}`}>
      <div className="bg-gray-50 dark:bg-gray-900 min-h-screen">
        <Header />
        <div className="flex">
          <Sidebar />
          <main className="flex-1 p-6">
            {children}
          </main>
        </div>
      </div>
    </div>
  );
};

// components/layout/Header.tsx
export const Header: React.FC = () => {
  const { systemStats } = useSystemStats();
  const { theme, toggleTheme } = useThemeStore();
  
  return (
    <header className="bg-white dark:bg-gray-800 shadow-sm border-b">
      <div className="px-6 py-4 flex justify-between items-center">
        <div className="flex items-center space-x-4">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            TabAgent Dashboard
          </h1>
          <StatusIndicator status={systemStats?.status} />
        </div>
        
        <div className="flex items-center space-x-4">
          <ThemeToggle theme={theme} onToggle={toggleTheme} />
          <NotificationBell />
          <UserMenu />
        </div>
      </div>
    </header>
  );
};
```

### Feature Components

```typescript
// components/features/logs/LogsViewer.tsx
export const LogsViewer: React.FC = () => {
  const [filters, setFilters] = useState<LogFilters>({
    level: 'all',
    source: 'all',
    timeRange: '1h',
  });
  
  const { logs } = useLogStream(filters);
  const [selectedLog, setSelectedLog] = useState<LogEntry | null>(null);
  
  return (
    <Card className="h-full">
      <CardHeader>
        <div className="flex justify-between items-center">
          <h2 className="text-xl font-semibold">System Logs</h2>
          <div className="flex space-x-2">
            <LogFilters filters={filters} onChange={setFilters} />
            <Button onClick={() => exportLogs(logs)}>
              Export
            </Button>
          </div>
        </div>
      </CardHeader>
      
      <CardContent className="h-96 overflow-hidden">
        <div className="h-full flex">
          <div className="flex-1 overflow-y-auto">
            <VirtualizedList
              items={logs}
              renderItem={(log) => (
                <LogEntry
                  key={log.id}
                  log={log}
                  onClick={() => setSelectedLog(log)}
                  isSelected={selectedLog?.id === log.id}
                />
              )}
            />
          </div>
          
          {selectedLog && (
            <div className="w-80 border-l pl-4">
              <LogDetails log={selectedLog} />
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

// components/features/models/ModelManager.tsx
export const ModelManager: React.FC = () => {
  const { data: models, isLoading } = useQuery(['models'], apiClient.listModels);
  const { data: metrics } = useModelMetrics();
  const loadModelMutation = useMutation(apiClient.loadModel);
  
  if (isLoading) return <LoadingSpinner />;
  
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">Model Management</h2>
        <Button onClick={() => setShowLoadModal(true)}>
          Load New Model
        </Button>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {models?.map(model => (
          <ModelCard
            key={model.id}
            model={model}
            metrics={metrics?.[model.id]}
            onLoad={() => loadModelMutation.mutate(model.id)}
            onUnload={() => unloadModelMutation.mutate(model.id)}
          />
        ))}
      </div>
      
      <ModelMetricsChart data={metrics} />
    </div>
  );
};
```

## Data Models

### TypeScript Type Definitions

```typescript
// types/api.ts
export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  services: {
    http: boolean;
    webrtc: boolean;
    native_messaging: boolean;
  };
}

export interface SystemInfo {
  version: string;
  uptime: number;
  cpu_usage: number;
  memory_usage: number;
  gpu_usage?: number;
  disk_usage: number;
  active_connections: number;
}

export interface PerformanceStats {
  requests_per_second: number;
  average_response_time: number;
  error_rate: number;
  inference_stats: {
    total_inferences: number;
    average_ttft: number;
    average_tokens_per_second: number;
  };
}

// types/logs.ts
export interface LogEntry {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  source: string;
  context: string;
  message: string;
  data?: any;
}

export interface LogFilters {
  level: string;
  source: string;
  context?: string;
  timeRange: string;
  search?: string;
}

// types/models.ts
export interface ModelInfo {
  id: string;
  name: string;
  type: 'language' | 'vision' | 'audio' | 'multimodal';
  status: 'loaded' | 'loading' | 'unloaded' | 'error';
  memory_usage?: number;
  parameters?: number;
  quantization?: string;
}

export interface ModelMetrics {
  inference_count: number;
  average_latency: number;
  tokens_per_second: number;
  memory_usage: number;
  gpu_utilization?: number;
}
```

## Error Handling

### Comprehensive Error Management

```typescript
// lib/error-handling.ts
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export const handleApiError = (error: unknown): string => {
  if (error instanceof ApiError) {
    switch (error.status) {
      case 404:
        return 'Resource not found';
      case 500:
        return 'Server error occurred';
      case 503:
        return 'Service temporarily unavailable';
      default:
        return error.message;
    }
  }
  
  if (error instanceof Error) {
    return error.message;
  }
  
  return 'An unexpected error occurred';
};

// Error boundary component
export const ErrorBoundary: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ReactErrorBoundary
      FallbackComponent={ErrorFallback}
      onError={(error, errorInfo) => {
        console.error('Dashboard error:', error, errorInfo);
        // Could send to error reporting service
      }}
    >
      {children}
    </ReactErrorBoundary>
  );
};
```

## Testing Strategy

### Component Testing

```typescript
// components/__tests__/LogsViewer.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { LogsViewer } from '../LogsViewer';

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('LogsViewer', () => {
  it('renders log entries correctly', async () => {
    render(<LogsViewer />, { wrapper: createWrapper() });
    
    await waitFor(() => {
      expect(screen.getByText('System Logs')).toBeInTheDocument();
    });
  });
  
  it('filters logs by level', async () => {
    render(<LogsViewer />, { wrapper: createWrapper() });
    
    // Test filter functionality
    const levelFilter = screen.getByLabelText('Log Level');
    fireEvent.change(levelFilter, { target: { value: 'error' } });
    
    await waitFor(() => {
      // Verify only error logs are shown
    });
  });
});
```

### Integration Testing

```typescript
// __tests__/integration/dashboard.test.tsx
describe('Dashboard Integration', () => {
  it('loads and displays system data correctly', async () => {
    // Mock API responses
    server.use(
      rest.get('/v1/health', (req, res, ctx) => {
        return res(ctx.json({ status: 'healthy' }));
      }),
      rest.get('/v1/stats', (req, res, ctx) => {
        return res(ctx.json({ requests_per_second: 100 }));
      })
    );
    
    render(<App />);
    
    await waitFor(() => {
      expect(screen.getByText('healthy')).toBeInTheDocument();
      expect(screen.getByText('100')).toBeInTheDocument();
    });
  });
});
```

## Performance Considerations

### Optimization Strategies

1. **Code Splitting**: Route-based and component-based code splitting
2. **Virtual Scrolling**: For large log lists and data tables
3. **Memoization**: React.memo and useMemo for expensive computations
4. **Debounced Updates**: For real-time data to prevent excessive re-renders
5. **Efficient State Management**: Zustand for minimal re-renders

### Build Optimization

```typescript
// vite.config.ts
export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          charts: ['recharts'],
          ui: ['@headlessui/react', 'framer-motion'],
        },
      },
    },
  },
  server: {
    proxy: {
      '/v1': 'http://localhost:3000',
      '/ws': {
        target: 'ws://localhost:3000',
        ws: true,
      },
    },
  },
});
```

## Integration with Rust Backend

### Static File Serving

```rust
// In Rust server router.rs
let dashboard_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .unwrap()
    .join("dashboard/dist");

if dashboard_dir.exists() {
    tracing::info!("📱 Serving React dashboard from {:?}", dashboard_dir);
    
    router = router
        // Serve React app
        .route_service("/", ServeFile::new(dashboard_dir.join("index.html")))
        .nest_service("/assets", ServeDir::new(dashboard_dir.join("assets")))
        // API routes take precedence
        .nest("/api", api_routes())
        .nest("/v1", v1_routes())
        // WebSocket routes for real-time updates
        .route("/ws/logs", get(logs_websocket))
        .route("/ws/system", get(system_websocket));
}
```

### Development Workflow

```bash
# Development (parallel processes)
# Terminal 1: Rust backend
cd Rust && cargo run --bin tabagent-server -- --mode all

# Terminal 2: React frontend (with proxy to Rust)
cd dashboard && npm run dev

# Production build
cd dashboard && npm run build
# Built files automatically served by Rust server
```

## Migration Path

### Phase 1: Foundation (Week 1)
1. Set up React + TypeScript + Vite project
2. Create basic layout and routing
3. Implement API client and basic data fetching
4. Create core UI components

### Phase 2: Core Features (Week 2)
1. System monitoring dashboard
2. Real-time log viewer
3. Model management interface
4. Basic charts and visualizations

### Phase 3: Advanced Features (Week 3)
1. Database explorer
2. Advanced filtering and search
3. Real-time WebSocket integration
4. Performance optimizations

### Phase 4: Polish (Week 4)
1. Comprehensive testing
2. Accessibility improvements
3. Mobile responsiveness
4. Documentation and deployment