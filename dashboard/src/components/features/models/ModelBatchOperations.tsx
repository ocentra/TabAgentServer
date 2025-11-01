import React, { useState } from 'react';
import { Button } from '../../ui/Button';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { ModelConfirmationDialog } from './ModelConfirmationDialog';
import type { ModelInfo } from '../../../types/models';

interface ModelBatchOperationsProps {
  selectedModels: Set<string>;
  models: ModelInfo[];
  onClearSelection: () => void;
  onBatchLoad: (modelIds: string[]) => Promise<void>;
  onBatchUnload: (modelIds: string[]) => Promise<void>;
  onBatchConfigure: (modelIds: string[], config: Record<string, unknown>) => Promise<void>;
  isLoading?: boolean;
}

export const ModelBatchOperations: React.FC<ModelBatchOperationsProps> = ({
  selectedModels,
  models,
  onClearSelection,
  onBatchLoad,
  onBatchUnload,
  onBatchConfigure,
  isLoading = false,
}) => {
  const [batchAction, setBatchAction] = useState<'load' | 'unload' | 'configure' | null>(null);
  const [showConfirmation, setShowConfirmation] = useState(false);

  // Get selected model objects
  const selectedModelObjects = models.filter(m => selectedModels.has(m.id));

  // Count models by status
  const statusCounts = selectedModelObjects.reduce((acc, model) => {
    acc[model.status] = (acc[model.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const canLoad = statusCounts.unloaded > 0 || statusCounts.error > 0;
  const canUnload = statusCounts.loaded > 0;

  const handleBatchAction = async (action: 'load' | 'unload' | 'configure') => {
    setBatchAction(action);
    setShowConfirmation(true);
  };

  const handleConfirmBatchAction = async () => {
    if (!batchAction) return;

    const modelIds = Array.from(selectedModels);

    try {
      switch (batchAction) {
        case 'load':
          const loadableModels = selectedModelObjects
            .filter(m => m.status === 'unloaded' || m.status === 'error')
            .map(m => m.id);
          await onBatchLoad(loadableModels);
          break;
        case 'unload':
          const unloadableModels = selectedModelObjects
            .filter(m => m.status === 'loaded')
            .map(m => m.id);
          await onBatchUnload(unloadableModels);
          break;
        case 'configure':
          // For now, apply default configuration
          await onBatchConfigure(modelIds, {
            temperature: 0.7,
            max_tokens: 2048,
          });
          break;
      }
    } catch (error) {
      console.error(`Batch ${batchAction} failed:`, error);
    } finally {
      setShowConfirmation(false);
      setBatchAction(null);
    }
  };



  if (selectedModels.size === 0) return null;

  return (
    <>
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <span className="text-sm font-medium text-blue-900 dark:text-blue-100">
                {selectedModels.size} model(s) selected
              </span>
              {Object.entries(statusCounts).map(([status, count]) => (
                <span
                  key={status}
                  className="px-2 py-1 text-xs rounded-full bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-200"
                >
                  {count} {status}
                </span>
              ))}
            </div>
          </div>

          <div className="flex items-center space-x-2">
            {/* Batch Action Selector */}
            <Select
              value=""
              onChange={(value: string) => {
                if (value) handleBatchAction(value as any);
              }}
              className="w-40"
            >
              <option value="" disabled>
                Batch Actions
              </option>
              {canLoad && (
                <option value="load">
                  Load Models ({statusCounts.unloaded || 0 + statusCounts.error || 0})
                </option>
              )}
              {canUnload && (
                <option value="unload">
                  Unload Models ({statusCounts.loaded || 0})
                </option>
              )}
              <option value="configure">
                Configure All ({selectedModels.size})
              </option>
            </Select>

            {/* Quick Action Buttons */}
            {canLoad && (
              <Button
                onClick={() => handleBatchAction('load')}
                size="sm"
                disabled={isLoading}
              >
                {isLoading ? <LoadingSpinner size="sm" /> : 'Load All'}
              </Button>
            )}

            {canUnload && (
              <Button
                onClick={() => handleBatchAction('unload')}
                variant="outline"
                size="sm"
                disabled={isLoading}
              >
                {isLoading ? <LoadingSpinner size="sm" /> : 'Unload All'}
              </Button>
            )}

            <Button
              onClick={onClearSelection}
              variant="outline"
              size="sm"
            >
              Clear Selection
            </Button>
          </div>
        </div>

        {/* Status Breakdown */}
        <div className="mt-3 pt-3 border-t border-blue-200 dark:border-blue-700">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div className="flex justify-between">
              <span className="text-blue-700 dark:text-blue-300">Total Memory:</span>
              <span className="font-medium text-blue-900 dark:text-blue-100">
                {(selectedModelObjects
                  .reduce((sum, m) => sum + (m.memory_usage || 0), 0) / 1024 / 1024 / 1024
                ).toFixed(1)} GB
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-blue-700 dark:text-blue-300">Avg Parameters:</span>
              <span className="font-medium text-blue-900 dark:text-blue-100">
                {selectedModelObjects.length > 0
                  ? (selectedModelObjects
                      .reduce((sum, m) => sum + (m.parameters || 0), 0) / selectedModelObjects.length / 1e9
                    ).toFixed(1) + 'B'
                  : '0B'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-blue-700 dark:text-blue-300">Types:</span>
              <span className="font-medium text-blue-900 dark:text-blue-100">
                {Array.from(new Set(selectedModelObjects.map(m => m.type))).length} unique
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-blue-700 dark:text-blue-300">Ready to Load:</span>
              <span className="font-medium text-blue-900 dark:text-blue-100">
                {(statusCounts.unloaded || 0) + (statusCounts.error || 0)}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Confirmation Dialog */}
      {showConfirmation && batchAction && (
        <ModelConfirmationDialog
          isOpen={showConfirmation}
          onClose={() => {
            setShowConfirmation(false);
            setBatchAction(null);
          }}
          onConfirm={handleConfirmBatchAction}
          model={{
            id: 'batch',
            name: `${selectedModels.size} Selected Models`,
            type: 'multimodal',
            status: 'unloaded',
            memory_usage: selectedModelObjects.reduce((sum, m) => sum + (m.memory_usage || 0), 0),
            parameters: selectedModelObjects.reduce((sum, m) => sum + (m.parameters || 0), 0),
          } as ModelInfo}
          action={batchAction === 'configure' ? 'load' : batchAction}
          isLoading={isLoading}
        />
      )}
    </>
  );
};