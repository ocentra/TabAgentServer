import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useDatabaseBackups, useCreateBackup } from '@/hooks/useDatabase';
import { formatBytes, formatDate } from '@/lib/utils';
import type { DatabaseBackupInfo } from '@/types/database';

interface DataManagementProps {
  className?: string;
}

interface DataImportProps {
  onImportComplete?: () => void;
}

interface DataValidationProps {
  onValidationComplete?: (results: any) => void;
}

interface BackupManagementProps {
  onBackupComplete?: () => void;
}

const DataImport: React.FC<DataImportProps> = ({ onImportComplete }) => {
  const [importFile, setImportFile] = useState<File | null>(null);
  const [importFormat, setImportFormat] = useState<'json' | 'csv' | 'graphml'>('json');
  const [importOptions, setImportOptions] = useState({
    validateData: true,
    skipDuplicates: true,
    createBackup: true,
  });
  const [isImporting, setIsImporting] = useState(false);
  const [importProgress, setImportProgress] = useState(0);
  const [importError, setImportError] = useState<string | null>(null);

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setImportFile(file);
      setImportError(null);
    }
  };

  const handleImport = async () => {
    if (!importFile) return;

    setIsImporting(true);
    setImportProgress(0);
    setImportError(null);

    try {
      const formData = new FormData();
      formData.append('file', importFile);
      formData.append('format', importFormat);
      formData.append('options', JSON.stringify(importOptions));

      // Simulate progress updates
      const progressInterval = setInterval(() => {
        setImportProgress(prev => Math.min(prev + 10, 90));
      }, 500);

      const response = await fetch('/v1/database/import', {
        method: 'POST',
        body: formData,
      });

      clearInterval(progressInterval);

      if (!response.ok) {
        throw new Error(`Import failed: ${response.statusText}`);
      }

      await response.json();
      setImportProgress(100);
      
      setTimeout(() => {
        setIsImporting(false);
        setImportFile(null);
        setImportProgress(0);
        onImportComplete?.();
      }, 1000);

    } catch (error) {
      setImportError(error instanceof Error ? error.message : 'Import failed');
      setIsImporting(false);
      setImportProgress(0);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Data Import</CardTitle>
        <CardDescription>
          Import data from external files into the database
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* File Selection */}
        <div>
          <label className="block text-sm font-medium mb-2">Select File</label>
          <Input
            type="file"
            accept=".json,.csv,.graphml"
            onChange={handleFileSelect}
            disabled={isImporting}
          />
          {importFile && (
            <div className="mt-2 text-sm text-muted-foreground">
              Selected: {importFile.name} ({formatBytes(importFile.size)})
            </div>
          )}
        </div>

        {/* Format Selection */}
        <div>
          <label className="block text-sm font-medium mb-2">Import Format</label>
          <Select
            value={importFormat}
            onChange={(value) => setImportFormat(value as 'json' | 'csv' | 'graphml')}
            disabled={isImporting}
          >
            <option value="json">JSON</option>
            <option value="csv">CSV</option>
            <option value="graphml">GraphML</option>
          </Select>
        </div>

        {/* Import Options */}
        <div>
          <label className="block text-sm font-medium mb-2">Options</label>
          <div className="space-y-2">
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={importOptions.validateData}
                onChange={(e) => setImportOptions(prev => ({ 
                  ...prev, 
                  validateData: e.target.checked 
                }))}
                disabled={isImporting}
                className="rounded border-gray-300"
              />
              <span className="text-sm">Validate data before import</span>
            </label>
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={importOptions.skipDuplicates}
                onChange={(e) => setImportOptions(prev => ({ 
                  ...prev, 
                  skipDuplicates: e.target.checked 
                }))}
                disabled={isImporting}
                className="rounded border-gray-300"
              />
              <span className="text-sm">Skip duplicate entries</span>
            </label>
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={importOptions.createBackup}
                onChange={(e) => setImportOptions(prev => ({ 
                  ...prev, 
                  createBackup: e.target.checked 
                }))}
                disabled={isImporting}
                className="rounded border-gray-300"
              />
              <span className="text-sm">Create backup before import</span>
            </label>
          </div>
        </div>

        {/* Import Progress */}
        {isImporting && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Importing...</span>
              <span className="text-sm text-muted-foreground">{importProgress}%</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div 
                className="bg-primary h-2 rounded-full transition-all duration-300"
                style={{ width: `${importProgress}%` }}
              />
            </div>
          </div>
        )}

        {/* Import Error */}
        {importError && (
          <div className="border border-destructive rounded-lg p-3 bg-destructive/5">
            <p className="text-sm text-destructive">{importError}</p>
          </div>
        )}

        {/* Import Button */}
        <Button 
          onClick={handleImport}
          disabled={!importFile || isImporting}
          className="w-full"
        >
          {isImporting ? (
            <>
              <LoadingSpinner size="sm" className="mr-2" />
              Importing...
            </>
          ) : (
            <>
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
              </svg>
              Import Data
            </>
          )}
        </Button>
      </CardContent>
    </Card>
  );
};

const DataValidation: React.FC<DataValidationProps> = ({ onValidationComplete }) => {
  const [isValidating, setIsValidating] = useState(false);
  const [validationResults, setValidationResults] = useState<any>(null);
  const [validationError, setValidationError] = useState<string | null>(null);

  const runValidation = async () => {
    setIsValidating(true);
    setValidationError(null);

    try {
      const response = await fetch('/v1/database/validate', {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Validation failed: ${response.statusText}`);
      }

      const results = await response.json();
      setValidationResults(results);
      onValidationComplete?.(results);

    } catch (error) {
      setValidationError(error instanceof Error ? error.message : 'Validation failed');
    } finally {
      setIsValidating(false);
    }
  };

  const runCleanup = async () => {
    if (!validationResults?.issues?.length) return;

    try {
      const response = await fetch('/v1/database/cleanup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          fix_orphaned_nodes: true,
          remove_duplicate_edges: true,
          update_invalid_properties: true,
        }),
      });

      if (!response.ok) {
        throw new Error(`Cleanup failed: ${response.statusText}`);
      }

      // Re-run validation after cleanup
      await runValidation();

    } catch (error) {
      setValidationError(error instanceof Error ? error.message : 'Cleanup failed');
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Data Validation & Cleanup</CardTitle>
        <CardDescription>
          Check database integrity and fix common issues
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Validation Controls */}
        <div className="flex space-x-2">
          <Button 
            onClick={runValidation}
            disabled={isValidating}
            variant="outline"
          >
            {isValidating ? (
              <>
                <LoadingSpinner size="sm" className="mr-2" />
                Validating...
              </>
            ) : (
              <>
                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Run Validation
              </>
            )}
          </Button>

          {validationResults?.issues?.length > 0 && (
            <Button onClick={runCleanup}>
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              Run Cleanup
            </Button>
          )}
        </div>

        {/* Validation Results */}
        {validationResults && (
          <div className="space-y-3">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="text-center p-3 border rounded-lg">
                <div className="text-2xl font-bold text-green-600">
                  {validationResults.valid_nodes || 0}
                </div>
                <div className="text-sm text-muted-foreground">Valid Nodes</div>
              </div>
              <div className="text-center p-3 border rounded-lg">
                <div className="text-2xl font-bold text-green-600">
                  {validationResults.valid_edges || 0}
                </div>
                <div className="text-sm text-muted-foreground">Valid Edges</div>
              </div>
              <div className="text-center p-3 border rounded-lg">
                <div className={`text-2xl font-bold ${
                  validationResults.issues?.length > 0 ? 'text-red-600' : 'text-green-600'
                }`}>
                  {validationResults.issues?.length || 0}
                </div>
                <div className="text-sm text-muted-foreground">Issues Found</div>
              </div>
            </div>

            {/* Issues List */}
            {validationResults.issues?.length > 0 && (
              <div>
                <h4 className="font-medium mb-2">Issues Found:</h4>
                <div className="space-y-2">
                  {validationResults.issues.map((issue: any, index: number) => (
                    <div key={index} className="border rounded-lg p-3 bg-yellow-50 dark:bg-yellow-900/20">
                      <div className="flex items-start justify-between">
                        <div>
                          <Badge variant="warning" className="mb-1">
                            {issue.type}
                          </Badge>
                          <p className="text-sm">{issue.description}</p>
                          {issue.affected_count && (
                            <p className="text-xs text-muted-foreground mt-1">
                              Affects {issue.affected_count} items
                            </p>
                          )}
                        </div>
                        <Badge variant={issue.severity === 'high' ? 'error' : 'warning'}>
                          {issue.severity}
                        </Badge>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Validation Summary */}
            <div className="border rounded-lg p-3 bg-muted/50">
              <h4 className="font-medium mb-2">Validation Summary</h4>
              <div className="text-sm space-y-1">
                <div>Last validated: {formatDate(validationResults.timestamp)}</div>
                <div>Validation time: {validationResults.duration_ms}ms</div>
                <div>Database health: 
                  <Badge 
                    variant={validationResults.issues?.length === 0 ? 'success' : 'warning'}
                    className="ml-2"
                  >
                    {validationResults.issues?.length === 0 ? 'Healthy' : 'Needs Attention'}
                  </Badge>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Validation Error */}
        {validationError && (
          <div className="border border-destructive rounded-lg p-3 bg-destructive/5">
            <p className="text-sm text-destructive">{validationError}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
};

const BackupManagement: React.FC<BackupManagementProps> = ({ onBackupComplete }) => {
  const [backupName, setBackupName] = useState('');
  const [backupType, setBackupType] = useState<'full' | 'incremental'>('full');
  
  const { data: backups, isLoading, refetch } = useDatabaseBackups();
  const createBackupMutation = useCreateBackup();

  const handleCreateBackup = async () => {
    if (!backupName.trim()) return;

    try {
      await createBackupMutation.mutateAsync({
        name: backupName.trim(),
        type: backupType,
      });
      
      setBackupName('');
      onBackupComplete?.();
      refetch();
    } catch (error) {
      console.error('Backup creation failed:', error);
    }
  };

  const handleRestoreBackup = async (backupId: string) => {
    if (!confirm('Are you sure you want to restore this backup? This will replace all current data.')) {
      return;
    }

    try {
      const response = await fetch(`/v1/database/backups/${backupId}/restore`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`Restore failed: ${response.statusText}`);
      }

      alert('Backup restored successfully');
      
    } catch (error) {
      alert(`Restore failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  const handleDeleteBackup = async (backupId: string) => {
    if (!confirm('Are you sure you want to delete this backup?')) {
      return;
    }

    try {
      const response = await fetch(`/v1/database/backups/${backupId}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        throw new Error(`Delete failed: ${response.statusText}`);
      }

      refetch();
      
    } catch (error) {
      alert(`Delete failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Backup & Restore</CardTitle>
        <CardDescription>
          Create and manage database backups
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Create Backup */}
        <div className="space-y-3">
          <h4 className="font-medium">Create New Backup</h4>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <Input
              placeholder="Backup name"
              value={backupName}
              onChange={(e) => setBackupName(e.target.value)}
              disabled={createBackupMutation.isPending}
            />
            <Select
              value={backupType}
              onChange={(value) => setBackupType(value as 'full' | 'incremental')}
              disabled={createBackupMutation.isPending}
            >
              <option value="full">Full Backup</option>
              <option value="incremental">Incremental Backup</option>
            </Select>
            <Button 
              onClick={handleCreateBackup}
              disabled={!backupName.trim() || createBackupMutation.isPending}
            >
              {createBackupMutation.isPending ? (
                <>
                  <LoadingSpinner size="sm" className="mr-2" />
                  Creating...
                </>
              ) : (
                <>
                  <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3-3m0 0l-3 3m3-3v12" />
                  </svg>
                  Create Backup
                </>
              )}
            </Button>
          </div>
        </div>

        {/* Backup List */}
        <div>
          <h4 className="font-medium mb-3">Existing Backups</h4>
          {isLoading ? (
            <div className="flex items-center justify-center h-32">
              <LoadingSpinner />
            </div>
          ) : backups?.length ? (
            <div className="space-y-2">
              {backups.map((backup: DatabaseBackupInfo) => (
                <div key={backup.id} className="border rounded-lg p-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="flex items-center space-x-2">
                        <span className="font-medium">{backup.name}</span>
                        <Badge variant={backup.type === 'full' ? 'default' : 'secondary'}>
                          {backup.type}
                        </Badge>
                        <Badge variant={
                          backup.status === 'completed' ? 'success' :
                          backup.status === 'in_progress' ? 'warning' : 'error'
                        }>
                          {backup.status}
                        </Badge>
                      </div>
                      <div className="text-sm text-muted-foreground mt-1">
                        {formatDate(backup.created_at)} • {formatBytes(backup.size_bytes)}
                      </div>
                    </div>
                    <div className="flex space-x-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleRestoreBackup(backup.id)}
                        disabled={backup.status !== 'completed'}
                      >
                        Restore
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDeleteBackup(backup.id)}
                        disabled={backup.status === 'in_progress'}
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              <p>No backups found</p>
              <p className="text-sm">Create your first backup to get started</p>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export const DataManagement: React.FC<DataManagementProps> = ({ className }) => {
  const [activeSection, setActiveSection] = useState<'import' | 'validation' | 'backup'>('import');

  const sections = [
    { id: 'import', label: 'Import Data', icon: '📥' },
    { id: 'validation', label: 'Validation & Cleanup', icon: '🔍' },
    { id: 'backup', label: 'Backup & Restore', icon: '💾' },
  ];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Section Navigation */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex space-x-2">
            {sections.map((section) => (
              <Button
                key={section.id}
                variant={activeSection === section.id ? "default" : "outline"}
                onClick={() => setActiveSection(section.id as any)}
              >
                <span className="mr-2">{section.icon}</span>
                {section.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Section Content */}
      {activeSection === 'import' && (
        <DataImport onImportComplete={() => console.log('Import completed')} />
      )}
      
      {activeSection === 'validation' && (
        <DataValidation onValidationComplete={(results) => console.log('Validation completed:', results)} />
      )}
      
      {activeSection === 'backup' && (
        <BackupManagement onBackupComplete={() => console.log('Backup completed')} />
      )}
    </div>
  );
};