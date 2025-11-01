# Features - Domain-Specific Components

**Feature modules for TabAgent dashboard functionality**

Organized by domain: system monitoring, model management, log analysis, and database exploration.

---

## Modules

### [`system/`](system/) - System Monitoring
- `SystemMonitor` - Main system overview
- `ResourceCharts` - CPU/GPU/Memory charts
- `MetricCard` - Individual metric display
- `NotificationBell` - Alert system
- `PerformanceDashboard` - Performance metrics

### [`models/`](models/) - Model Management  
- `ModelManager` - Main model interface
- `ModelCard` - Individual model display
- `ModelMetrics` - Performance analytics
- `ModelConfigurationDialog` - Model settings
- `ModelBatchOperations` - Multi-model actions

### [`logs/`](logs/) - Log Analysis
- `LogsViewer` - Main log interface with virtualization
- `LogEntry` - Individual log entry display
- `LogFilters` - Advanced filtering controls
- `LogAnalytics` - Statistics and trends
- `LogExport` - Download and export functionality

### [`database/`](database/) - Database Exploration
- `DatabaseExplorer` - Overview and statistics
- `NodeViewer` - Browse database nodes
- `SearchInterface` - Advanced search with filters
- `KnowledgeGraphVisualization` - Interactive D3.js graph
- `DataManagement` - Import/export/backup tools

---

## Architecture

```
Feature Components
├── Domain Logic (hooks, utilities)
├── UI Components (forms, displays)
├── API Integration (React Query)
└── Real-time Updates (WebSocket)
```

**Patterns**:
- Each feature is self-contained
- Shared UI components from `../ui/`
- API calls via custom hooks
- Consistent error handling

---

## Usage

```tsx
// System monitoring
import { SystemMonitor } from '@/components/features/system';
<SystemMonitor />

// Model management
import { ModelManager } from '@/components/features/models';
<ModelManager />

// Log analysis
import { LogsViewer } from '@/components/features/logs';
<LogsViewer autoScroll={true} />

// Database exploration
import { DatabaseExplorer } from '@/components/features/database';
<DatabaseExplorer />
```

---

## Features

✅ **Real-time Updates** - Live data via WebSocket  
✅ **Advanced Filtering** - Search, sort, filter all data  
✅ **Export Functionality** - Download data in multiple formats  
✅ **Interactive Controls** - Manage models, clear logs, etc.  
✅ **Responsive Design** - Works on all devices  
✅ **Error Handling** - Graceful degradation on API failures  

---

## Dependencies

- **React Query** - Data fetching and caching
- **WebSocket** - Real-time updates  
- **D3.js** - Knowledge graph visualization
- **React Window** - Virtualized scrolling for logs
- **Recharts** - Chart components