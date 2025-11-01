import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Select } from '@/components/ui/Select';
import { Input } from '@/components/ui/Input';
import { useExportDatabase } from '@/hooks/useDatabase';
import { formatBytes, formatDate } from '@/lib/utils';
import type { DatabaseExportOptions } from '@/types/database';

interface DataExportProps {
  className?: string;
  onClose?: () => void;
}

export const DataExport: React.FC<DataExportProps> = ({ className, onClose }) => {
  const [exportOptions, setExportOptions] = useState<DatabaseExportOptions>({
    format: 'json',
    compression: true,
  });
  const [selectedTypes, setSelectedTypes] = useState<string[]>([]);
  const [dateRange, setDateRange] = useState<{ start: string; end: string }>({
    start: '',
    end: '',
  });

  const exportMutation = useExportDatabase();

  const exportFormats = [
    { 
      value: 'json', 
      label: 'JSON', 
      description: 'Structured data format, good for programmatic use',
      extension: '.json'
    },
    { 
      value: 'csv', 
      label: 'CSV', 
      description: 'Comma-separated values, good for spreadsheets',
      extension: '.csv'
    },
    { 
      value: 'graphml', 
      label: 'GraphML', 
      description: 'Graph markup language for network analysis',
      extension: '.graphml'
    },
    { 
      value: 'cypher', 
      label: 'Cypher', 
      description: 'Neo4j Cypher queries for graph databases',
      extension: '.cypher'
    },
  ];

  const nodeTypes = [
    { value: 'conversation', label: 'Conversations' },
    { value: 'message', label: 'Messages' },
    { value: 'document', label: 'Documents' },
    { value: 'entity', label: 'Entities' },
    { value: 'embedding', label: 'Embeddings' },
  ];

  const handleExport = async () => {
    const options: DatabaseExportOptions = {
      ...exportOptions,
      include_types: selectedTypes.length > 0 ? selectedTypes : undefined,
      date_range: (dateRange.start || dateRange.end) ? {
        start: dateRange.start || undefined,
        end: dateRange.end || undefined,
      } : undefined,
    };

    try {
      const blob = await exportMutation.mutateAsync(options);
      
      // Create download link
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      
      const format = exportFormats.find(f => f.value === options.format);
      const timestamp = new Date().toISOString().split('T')[0];
      link.download = `tabagent-database-${timestamp}${format?.extension || '.json'}${options.compression ? '.gz' : ''}`;
      
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      
      onClose?.();
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  const toggleType = (type: string) => {
    setSelectedTypes(prev => 
      prev.includes(type) 
        ? prev.filter(t => t !== type)
        : [...prev, type]
    );
  };

  const estimatedSize = () => {
    // Rough estimation based on selected options
    let baseSize = 1024 * 1024; // 1MB base
    
    if (selectedTypes.length === 0) {
      baseSize *= 5; // All types
    } else {
      baseSize *= selectedTypes.length;
    }
    
    if (exportOptions.format === 'json') baseSize *= 1.5;
    if (exportOptions.format === 'graphml') baseSize *= 2;
    if (exportOptions.compression) baseSize *= 0.3;
    
    return Math.round(baseSize);
  };

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Export Database</CardTitle>
            <CardDescription>
              Download database content in various formats
            </CardDescription>
          </div>
          {onClose && (
            <Button variant="ghost" size="sm" onClick={onClose}>
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </Button>
          )}
        </div>
      </CardHeader>
      
      <CardContent className="space-y-6">
        {/* Export Format */}
        <div>
          <label className="block text-sm font-medium mb-3">Export Format</label>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {exportFormats.map((format) => (
              <div
                key={format.value}
                className={`
                  border rounded-lg p-3 cursor-pointer transition-all
                  ${exportOptions.format === format.value 
                    ? 'border-primary bg-primary/5' 
                    : 'hover:border-primary/50'
                  }
                `}
                onClick={() => setExportOptions(prev => ({ ...prev, format: format.value as any }))}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="font-medium">{format.label}</span>
                  <Badge variant="outline">{format.extension}</Badge>
                </div>
                <p className="text-sm text-muted-foreground">{format.description}</p>
              </div>
            ))}
          </div>
        </div>

        {/* Content Types */}
        <div>
          <label className="block text-sm font-medium mb-3">
            Content Types
            <span className="text-muted-foreground font-normal ml-2">
              (leave empty to export all types)
            </span>
          </label>
          <div className="flex flex-wrap gap-2">
            {nodeTypes.map((type) => (
              <Button
                key={type.value}
                variant={selectedTypes.includes(type.value) ? "default" : "outline"}
                size="sm"
                onClick={() => toggleType(type.value)}
              >
                {type.label}
              </Button>
            ))}
          </div>
          {selectedTypes.length > 0 && (
            <div className="mt-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSelectedTypes([])}
              >
                Clear selection
              </Button>
            </div>
          )}
        </div>

        {/* Date Range */}
        <div>
          <label className="block text-sm font-medium mb-3">
            Date Range
            <span className="text-muted-foreground font-normal ml-2">
              (optional)
            </span>
          </label>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs text-muted-foreground mb-1">From</label>
              <Input
                type="date"
                value={dateRange.start}
                onChange={(e) => setDateRange(prev => ({ ...prev, start: e.target.value }))}
              />
            </div>
            <div>
              <label className="block text-xs text-muted-foreground mb-1">To</label>
              <Input
                type="date"
                value={dateRange.end}
                onChange={(e) => setDateRange(prev => ({ ...prev, end: e.target.value }))}
              />
            </div>
          </div>
        </div>

        {/* Export Options */}
        <div>
          <label className="block text-sm font-medium mb-3">Options</label>
          <div className="space-y-2">
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={exportOptions.compression}
                onChange={(e) => setExportOptions(prev => ({ 
                  ...prev, 
                  compression: e.target.checked 
                }))}
                className="rounded border-gray-300"
              />
              <span className="text-sm">Compress output (recommended for large exports)</span>
            </label>
          </div>
        </div>

        {/* Export Summary */}
        <div className="border rounded-lg p-4 bg-muted/50">
          <h4 className="font-medium mb-2">Export Summary</h4>
          <div className="space-y-1 text-sm">
            <div className="flex justify-between">
              <span>Format:</span>
              <Badge variant="outline">
                {exportFormats.find(f => f.value === exportOptions.format)?.label}
              </Badge>
            </div>
            <div className="flex justify-between">
              <span>Content Types:</span>
              <span className="text-muted-foreground">
                {selectedTypes.length === 0 ? 'All types' : `${selectedTypes.length} selected`}
              </span>
            </div>
            <div className="flex justify-between">
              <span>Date Range:</span>
              <span className="text-muted-foreground">
                {dateRange.start || dateRange.end 
                  ? `${dateRange.start || 'Beginning'} to ${dateRange.end || 'Now'}`
                  : 'All time'
                }
              </span>
            </div>
            <div className="flex justify-between">
              <span>Estimated Size:</span>
              <span className="text-muted-foreground">
                ~{formatBytes(estimatedSize())}
              </span>
            </div>
          </div>
        </div>

        {/* Export Actions */}
        <div className="flex space-x-3">
          <Button 
            onClick={handleExport}
            disabled={exportMutation.isLoading}
            className="flex-1"
          >
            {exportMutation.isLoading ? (
              <>
                <LoadingSpinner size="sm" className="mr-2" />
                Exporting...
              </>
            ) : (
              <>
                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4-4m0 0L8 8m4-4v12" />
                </svg>
                Export Data
              </>
            )}
          </Button>
          
          {onClose && (
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
          )}
        </div>

        {/* Export Error */}
        {exportMutation.error && (
          <div className="border border-destructive rounded-lg p-3 bg-destructive/5">
            <p className="text-sm text-destructive">
              Export failed: {exportMutation.error instanceof Error 
                ? exportMutation.error.message 
                : 'Unknown error occurred'
              }
            </p>
          </div>
        )}

        {/* Export Tips */}
        <div className="text-xs text-muted-foreground space-y-1">
          <p><strong>Tips:</strong></p>
          <ul className="list-disc list-inside space-y-1 ml-2">
            <li>JSON format preserves all data structure and relationships</li>
            <li>CSV format is best for tabular analysis but loses graph structure</li>
            <li>GraphML format is ideal for network analysis tools</li>
            <li>Enable compression for large datasets to reduce download size</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  );
};