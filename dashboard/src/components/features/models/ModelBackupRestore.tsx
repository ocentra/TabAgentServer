import React, { useState } from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo } from '../../../types/models';

interface BackupInfo {
  id: string;
  modelId: string;
  modelName: string;
  timestamp: string;
  size: number;
  version: string;
  checksum: string;
  metadata: {
    parameters: number;
    quantization?: string;
    config: Record<string, unknown>;
  };
}

interface ModelBackupRestoreProps {
  model: ModelInfo;
  backups?: BackupInfo[];
  onCreateBackup: (modelId: string, name?: string) => Promise<void>;
  onRestoreBackup: (backupId: string) => Promise<void>;
  onDeleteBackup: (backupId: string) => Promise<void>;
  isLoading?: boolean;
  className?: string;
}

export const ModelBackupRestore: React.FC<ModelBackupRestoreProps> = ({
  model,
  backups = [],
  onCreateBackup,
  onRestoreBackup,
  onDeleteBackup,
  isLoading = false,
  className = '',
}) => {
  const [backupName, setBackupName] = useState('');
  const [selectedBackup, setSelectedBackup] = useState<string | null>(null);
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);

  const handleCreateBackup = async () => {
    setIsCreatingBackup(true);
    try {
      await onCreateBackup(model.id, backupName.trim() || undefined);
      setBackupName('');
    } catch (error) {
      console.error('Failed to create backup:', error);
    } finally {
      setIsCreatingBackup(false);
    }
  };

  const handleRestoreBackup = async (backupId: string) => {
    setIsRestoring(true);
    try {
      await onRestoreBackup(backupId);
      setSelectedBackup(null);
    } catch (error) {
      console.error('Failed to restore backup:', error);
    } finally {
      setIsRestoring(false);
    }
  };

  const handleDeleteBackup = async (backupId: string) => {
    try {
      await onDeleteBackup(backupId);
    } catch (error) {
      console.error('Failed to delete backup:', error);
    }
  };

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatDate = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  // Mock backups for demonstration
  const mockBackups: BackupInfo[] = [
    {
      id: 'backup-1',
      modelId: model.id,
      modelName: model.name,
      timestamp: new Date(Date.now() - 86400000).toISOString(), // 1 day ago
      size: 2500000000, // 2.5GB
      version: '1.0.0',
      checksum: 'sha256:abc123...',
      metadata: {
        parameters: model.parameters || 0,
        quantization: model.quantization,
        config: { temperature: 0.7, max_tokens: 2048 },
      },
    },
    {
      id: 'backup-2',
      modelId: model.id,
      modelName: model.name,
      timestamp: new Date(Date.now() - 604800000).toISOString(), // 1 week ago
      size: 2600000000, // 2.6GB
      version: '0.9.0',
      checksum: 'sha256:def456...',
      metadata: {
        parameters: model.parameters || 0,
        quantization: 'none',
        config: { temperature: 0.8, max_tokens: 1024 },
      },
    },
  ];

  const displayBackups = backups.length > 0 ? backups : mockBackups;

  return (
    <Card className={className}>
      <CardHeader>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Backup & Restore
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
      </CardHeader>

      <CardContent>
        <div className="space-y-6">
          {/* Create Backup */}
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-3">Create Backup</h4>
            <div className="flex space-x-3">
              <Input
                placeholder="Backup name (optional)"
                value={backupName}
                onChange={(e) => setBackupName(e.target.value)}
                className="flex-1"
              />
              <Button
                onClick={handleCreateBackup}
                disabled={isCreatingBackup || model.status !== 'loaded'}
              >
                {isCreatingBackup ? (
                  <>
                    <LoadingSpinner size="sm" className="mr-2" />
                    Creating...
                  </>
                ) : (
                  'Create Backup'
                )}
              </Button>
            </div>
            {model.status !== 'loaded' && (
              <p className="text-sm text-yellow-600 dark:text-yellow-400 mt-2">
                Model must be loaded to create a backup
              </p>
            )}
          </div>

          {/* Backup List */}
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-3">
              Available Backups ({displayBackups.length})
            </h4>
            
            {displayBackups.length === 0 ? (
              <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                <div className="text-4xl mb-2">📦</div>
                <p className="text-lg">No backups available</p>
                <p className="text-sm mt-1">Create your first backup to get started</p>
              </div>
            ) : (
              <div className="space-y-3">
                {displayBackups.map((backup) => (
                  <div
                    key={backup.id}
                    className={`border rounded-lg p-4 transition-all ${
                      selectedBackup === backup.id
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-2">
                          <h5 className="font-medium text-gray-900 dark:text-white">
                            Backup {backup.version}
                          </h5>
                          <span className="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded">
                            {formatBytes(backup.size)}
                          </span>
                        </div>
                        
                        <div className="grid grid-cols-2 gap-4 text-sm text-gray-600 dark:text-gray-400 mb-3">
                          <div>
                            <span className="font-medium">Created:</span> {formatDate(backup.timestamp)}
                          </div>
                          <div>
                            <span className="font-medium">Parameters:</span> {(backup.metadata.parameters / 1e9).toFixed(1)}B
                          </div>
                          <div>
                            <span className="font-medium">Quantization:</span> {backup.metadata.quantization || 'None'}
                          </div>
                          <div>
                            <span className="font-medium">Checksum:</span> {backup.checksum.substring(0, 16)}...
                          </div>
                        </div>

                        {/* Backup Metadata */}
                        <details className="text-sm">
                          <summary className="cursor-pointer text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300">
                            Show configuration
                          </summary>
                          <pre className="mt-2 p-2 bg-gray-50 dark:bg-gray-800 rounded text-xs overflow-x-auto">
                            {JSON.stringify(backup.metadata.config, null, 2)}
                          </pre>
                        </details>
                      </div>

                      <div className="flex flex-col space-y-2 ml-4">
                        <Button
                          size="sm"
                          onClick={() => handleRestoreBackup(backup.id)}
                          disabled={isRestoring || isLoading}
                        >
                          {isRestoring && selectedBackup === backup.id ? (
                            <>
                              <LoadingSpinner size="sm" className="mr-1" />
                              Restoring...
                            </>
                          ) : (
                            'Restore'
                          )}
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => setSelectedBackup(selectedBackup === backup.id ? null : backup.id)}
                        >
                          {selectedBackup === backup.id ? 'Deselect' : 'Select'}
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => handleDeleteBackup(backup.id)}
                          className="text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
                        >
                          Delete
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Backup Information */}
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
            <div className="flex items-start space-x-2">
              <span className="text-blue-600 dark:text-blue-400 mt-0.5">ℹ️</span>
              <div className="text-sm text-blue-800 dark:text-blue-200">
                <p className="font-medium mb-1">Backup Information:</p>
                <ul className="space-y-1 text-xs">
                  <li>• Backups include model weights, configuration, and metadata</li>
                  <li>• Restoring a backup will replace the current model state</li>
                  <li>• Backup integrity is verified using checksums</li>
                  <li>• Large models may take several minutes to backup/restore</li>
                  <li>• Backups are stored locally and can be exported manually</li>
                </ul>
              </div>
            </div>
          </div>

          {/* Storage Usage */}
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-2">Storage Usage</h4>
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div className="text-center p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <div className="font-medium text-gray-900 dark:text-white">
                  {formatBytes(displayBackups.reduce((sum, b) => sum + b.size, 0))}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Total Backups</div>
              </div>
              <div className="text-center p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <div className="font-medium text-gray-900 dark:text-white">
                  {displayBackups.length}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Backup Count</div>
              </div>
              <div className="text-center p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <div className="font-medium text-gray-900 dark:text-white">
                  {displayBackups.length > 0 
                    ? formatBytes(displayBackups.reduce((sum, b) => sum + b.size, 0) / displayBackups.length)
                    : '0 B'}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Avg Size</div>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};