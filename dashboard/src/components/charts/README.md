# Charts - Visualization Components

**Reusable chart library for TabAgent dashboard**

Built with Recharts, supports real-time updates, theming, and export functionality.

---

## Components

### Basic Charts
- `LineChart` - Time-series data with multiple series
- `BarChart` - Categorical data (horizontal/vertical)
- `PieChart` - Distribution with donut mode support
- `AreaChart` - Filled area charts with gradients

### Advanced Charts
- `RealTimeChart` - Live data streaming with controls
- `ChartExport` - Export charts as images/data

### Dashboard-Specific
- `SystemResourceChart` - Real-time system monitoring
- `ModelPerformanceChart` - Model analytics and comparison
- `LogAnalyticsChart` - Log pattern analysis

---

## Usage

```tsx
import { LineChart, RealTimeChart } from '@/components/charts';

// Basic line chart
<LineChart
  data={timeSeriesData}
  series={[
    { key: 'cpu', name: 'CPU Usage', color: '#3b82f6' },
    { key: 'memory', name: 'Memory', color: '#10b981' }
  ]}
  height={300}
/>

// Real-time chart with live updates
<RealTimeChart
  title="System Metrics"
  series={series}
  dataSource={fetchSystemMetrics}
  updateInterval={5000}
/>
```

---

## Features

✅ **Responsive Design** - Works on all screen sizes  
✅ **Dark/Light Theme** - Automatic theme switching  
✅ **Interactive** - Tooltips, legends, zoom/pan  
✅ **Export** - Save as PNG/SVG, export data as CSV/JSON  
✅ **Real-time** - Live data streaming with controls  
✅ **TypeScript** - Full type safety  

---

## Chart Types

| Component | Use Case | Features |
|-----------|----------|----------|
| LineChart | Time-series trends | Multiple series, smooth curves |
| BarChart | Categorical comparison | Horizontal/vertical, stacking |
| PieChart | Distribution/percentages | Donut mode, center labels |
| AreaChart | Volume over time | Gradients, stacking support |
| RealTimeChart | Live monitoring | Start/stop/pause controls |

---

## Props Interface

```typescript
interface LineChartProps {
  data: LineChartDataPoint[];
  series: LineChartSeries[];
  height?: number;
  showGrid?: boolean;
  showLegend?: boolean;
  xAxisFormatter?: (value: any) => string;
  yAxisFormatter?: (value: any) => string;
}
```

All charts follow consistent prop patterns for easy usage.