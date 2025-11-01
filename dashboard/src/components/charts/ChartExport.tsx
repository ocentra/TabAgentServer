import React, { useRef } from 'react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';

export interface ChartExportProps {
  chartRef?: React.RefObject<HTMLDivElement>;
  data?: any[];
  filename?: string;
  title?: string;
  className?: string;
}

export const ChartExport: React.FC<ChartExportProps> = ({
  chartRef,
  data = [],
  filename = 'chart-data',
  title = 'Export Chart',
  className = '',
}) => {
  const downloadRef = useRef<HTMLAnchorElement>(null);

  const exportAsImage = async (format: 'png' | 'jpeg' | 'svg') => {
    if (!chartRef?.current) {
      console.error('Chart reference not available');
      return;
    }

    try {
      // Find the SVG element within the chart
      const svgElement = chartRef.current.querySelector('svg');
      if (!svgElement) {
        console.error('SVG element not found in chart');
        return;
      }

      if (format === 'svg') {
        // Export as SVG
        const svgData = new XMLSerializer().serializeToString(svgElement);
        const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
        const url = URL.createObjectURL(svgBlob);
        
        if (downloadRef.current) {
          downloadRef.current.href = url;
          downloadRef.current.download = `${filename}.svg`;
          downloadRef.current.click();
        }
        
        URL.revokeObjectURL(url);
      } else {
        // Export as PNG or JPEG
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const svgRect = svgElement.getBoundingClientRect();
        canvas.width = svgRect.width * 2; // Higher resolution
        canvas.height = svgRect.height * 2;
        ctx.scale(2, 2);

        // Create image from SVG
        const svgData = new XMLSerializer().serializeToString(svgElement);
        const img = new Image();
        
        img.onload = () => {
          // Fill background with white for JPEG
          if (format === 'jpeg') {
            ctx.fillStyle = 'white';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
          }
          
          ctx.drawImage(img, 0, 0);
          
          canvas.toBlob((blob) => {
            if (blob && downloadRef.current) {
              const url = URL.createObjectURL(blob);
              downloadRef.current.href = url;
              downloadRef.current.download = `${filename}.${format}`;
              downloadRef.current.click();
              URL.revokeObjectURL(url);
            }
          }, `image/${format}`, 0.9);
        };

        const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
        const url = URL.createObjectURL(svgBlob);
        img.src = url;
      }
    } catch (error) {
      console.error('Error exporting chart as image:', error);
    }
  };

  const exportAsCSV = () => {
    if (!data.length) {
      console.error('No data available for CSV export');
      return;
    }

    try {
      // Get all unique keys from the data
      const keys = Array.from(new Set(data.flatMap(Object.keys)));
      
      // Create CSV header
      const csvHeader = keys.join(',');
      
      // Create CSV rows
      const csvRows = data.map(row => 
        keys.map(key => {
          const value = row[key];
          // Handle values that might contain commas or quotes
          if (typeof value === 'string' && (value.includes(',') || value.includes('"'))) {
            return `"${value.replace(/"/g, '""')}"`;
          }
          return value ?? '';
        }).join(',')
      );
      
      const csvContent = [csvHeader, ...csvRows].join('\n');
      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      
      if (downloadRef.current) {
        downloadRef.current.href = url;
        downloadRef.current.download = `${filename}.csv`;
        downloadRef.current.click();
      }
      
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Error exporting data as CSV:', error);
    }
  };

  const exportAsJSON = () => {
    if (!data.length) {
      console.error('No data available for JSON export');
      return;
    }

    try {
      const jsonContent = JSON.stringify(data, null, 2);
      const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      
      if (downloadRef.current) {
        downloadRef.current.href = url;
        downloadRef.current.download = `${filename}.json`;
        downloadRef.current.click();
      }
      
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Error exporting data as JSON:', error);
    }
  };

  const copyToClipboard = async () => {
    if (!data.length) {
      console.error('No data available to copy');
      return;
    }

    try {
      const jsonContent = JSON.stringify(data, null, 2);
      await navigator.clipboard.writeText(jsonContent);
      
      // You could add a toast notification here
      console.log('Data copied to clipboard');
    } catch (error) {
      console.error('Error copying data to clipboard:', error);
    }
  };

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>
          Export chart data and images in various formats
        </CardDescription>
      </CardHeader>
      
      <CardContent className="space-y-4">
        {/* Image Export */}
        <div>
          <h4 className="font-medium mb-2">Export as Image</h4>
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => exportAsImage('png')}
              disabled={!chartRef?.current}
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              PNG
            </Button>
            
            <Button
              size="sm"
              variant="outline"
              onClick={() => exportAsImage('jpeg')}
              disabled={!chartRef?.current}
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              JPEG
            </Button>
            
            <Button
              size="sm"
              variant="outline"
              onClick={() => exportAsImage('svg')}
              disabled={!chartRef?.current}
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              SVG
            </Button>
          </div>
        </div>

        {/* Data Export */}
        <div>
          <h4 className="font-medium mb-2">Export Data</h4>
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={exportAsCSV}
              disabled={!data.length}
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              CSV
            </Button>
            
            <Button
              size="sm"
              variant="outline"
              onClick={exportAsJSON}
              disabled={!data.length}
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              JSON
            </Button>
            
            <Button
              size="sm"
              variant="outline"
              onClick={copyToClipboard}
              disabled={!data.length}
            >
              <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              Copy
            </Button>
          </div>
        </div>

        {/* Status */}
        <div className="text-sm text-muted-foreground">
          {data.length > 0 ? (
            <span>{data.length} data points available for export</span>
          ) : (
            <span>No data available for export</span>
          )}
        </div>
      </CardContent>
      
      {/* Hidden download link */}
      <a
        ref={downloadRef}
        style={{ display: 'none' }}
        href="#"
        download=""
      >
        Download
      </a>
    </Card>
  );
};