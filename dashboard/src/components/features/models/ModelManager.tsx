import React, { useState, useMemo } from 'react';
import { Card, CardContent, CardHeader } from '../../ui/Card';

import { Input } from '../../ui/Input';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { ModelCard } from './ModelCard';
import { ModelConfirmationDialog } from './ModelConfirmationDialog';
import { ModelConfigurationDialog } from './ModelConfigurationDialog';
import { ModelBatchOperations } from './ModelBatchOperations';
import { useModels, useModelMetrics, useLoadModel, useUnloadModel } from '../../../hooks/useApi';
import type { ModelInfo } from '../../../types/models';

interface ModelManagerProps {
  className?: string;
}

export const ModelManager: React.FC<ModelManagerProps> = ({ className }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [typeFilter, setTypeFilter] = useState<string>('all');
  const [selectedModels, setSelectedModels] = useState<Set<string>>(new Set());
  const [loadingModels, setLoadingModels] = useState<Set<string>>(new Set());
  
  // Dialog states
  const [confirmationDialog, setConfirmationDialog] = useState<{
    isOpen: boolean;
    model: ModelInfo | null;
    action: 'load' | 'unload' | 'delete';
  }>({ isOpen: false, model: null, action: 'load' });
  
  const [configurationDialog, setConfigurationDialog] = useState<{
    isOpen: boolean;
    model: ModelInfo | null;
  }>({ isOpen: false, model: null });

  // API hooks
  const { data: models, isLoading: modelsLoading, error: modelsError } = useModels();
  const { data: metrics } = useModelMetrics();
  const loadModelMutation = useLoadModel();
  const unloadModelMutation = useUnloadModel();

  // Filter and search models
  const filteredModels = useMemo(() => {
    if (!models) return [];

    return models.filter((model: ModelInfo) => {
      const matchesSearch = model.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           model.id.toLowerCase().includes(searchTerm.toLowerCase());
      const matchesStatus = statusFilter === 'all' || model.status === statusFilter;
      const matchesType = typeFilter === 'all' || model.type === typeFilter;

      return matchesSearch && matchesStatus && matchesType;
    });
  }, [models, searchTerm, statusFilter, typeFilter]);

  // Group models by status
  const modelsByStatus = useMemo(() => {
    const groups = {
      loaded: [] as ModelInfo[],
      loading: [] as ModelInfo[],
      unloaded: [] as ModelInfo[],
      error: [] as ModelInfo[],
    };

    filteredModels.forEach((model: ModelInfo) => {
      groups[model.status].push(model);
    });

    return groups;
  }, [filteredModels]);

  // Statistics
  const stats = useMemo(() => {
    if (!models) return { total: 0, loaded: 0, loading: 0, unloaded: 0, error: 0 };

    return models.reduce((acc, model: ModelInfo) => {
      acc.total++;
      acc[model.status]++;
      return acc;
    }, { total: 0, loaded: 0, loading: 0, unloaded: 0, error: 0 });
  }, [models]);

  // Handle model operations with confirmation
  const handleLoadModel = (modelId: string) => {
    const model = models?.find(m => m.id === modelId);
    if (model) {
      setConfirmationDialog({
        isOpen: true,
        model,
        action: 'load',
      });
    }
  };

  const handleUnloadModel = (modelId: string) => {
    const model = models?.find(m => m.id === modelId);
    if (model) {
      setConfirmationDialog({
        isOpen: true,
        model,
        action: 'unload',
      });
    }
  };

  const handleConfigureModel = (modelId: string) => {
    const model = models?.find(m => m.id === modelId);
    if (model) {
      setConfigurationDialog({
        isOpen: true,
        model,
      });
    }
  };

  // Execute confirmed operations
  const executeModelOperation = async (modelId: string, action: 'load' | 'unload') => {
    setLoadingModels(prev => new Set(prev).add(modelId));
    try {
      if (action === 'load') {
        await loadModelMutation.mutateAsync({ model_id: modelId });
      } else {
        await unloadModelMutation.mutateAsync({ model_id: modelId });
      }
    } catch (error) {
      console.error(`Failed to ${action} model:`, error);
    } finally {
      setLoadingModels(prev => {
        const newSet = new Set(prev);
        newSet.delete(modelId);
        return newSet;
      });
      setConfirmationDialog({ isOpen: false, model: null, action: 'load' });
    }
  };

  // Handle model configuration save
  const handleSaveConfiguration = async (config: any) => {
    if (!configurationDialog.model) return;
    
    try {
      // TODO: Implement API call to save model configuration
      console.log('Saving configuration for', configurationDialog.model.id, config);
      
      // For now, just close the dialog
      setConfigurationDialog({ isOpen: false, model: null });
    } catch (error) {
      console.error('Failed to save configuration:', error);
    }
  };

  // Batch operations
  const handleSelectModel = (modelId: string, selected: boolean) => {
    setSelectedModels(prev => {
      const newSet = new Set(prev);
      if (selected) {
        newSet.add(modelId);
      } else {
        newSet.delete(modelId);
      }
      return newSet;
    });
  };

  const handleSelectAll = () => {
    if (selectedModels.size === filteredModels.length) {
      setSelectedModels(new Set());
    } else {
      setSelectedModels(new Set(filteredModels.map(m => m.id)));
    }
  };

  // Batch operations
  const handleBatchLoad = async (modelIds: string[]) => {
    for (const modelId of modelIds) {
      setLoadingModels(prev => new Set(prev).add(modelId));
      try {
        await loadModelMutation.mutateAsync({ model_id: modelId });
      } catch (error) {
        console.error('Failed to load model:', modelId, error);
      } finally {
        setLoadingModels(prev => {
          const newSet = new Set(prev);
          newSet.delete(modelId);
          return newSet;
        });
      }
    }
    setSelectedModels(new Set());
  };

  const handleBatchUnload = async (modelIds: string[]) => {
    for (const modelId of modelIds) {
      setLoadingModels(prev => new Set(prev).add(modelId));
      try {
        await unloadModelMutation.mutateAsync({ model_id: modelId });
      } catch (error) {
        console.error('Failed to unload model:', modelId, error);
      } finally {
        setLoadingModels(prev => {
          const newSet = new Set(prev);
          newSet.delete(modelId);
          return newSet;
        });
      }
    }
    setSelectedModels(new Set());
  };

  const handleBatchConfigure = async (modelIds: string[], config: Record<string, unknown>) => {
    // TODO: Implement batch configuration API call
    console.log('Batch configure models:', modelIds, config);
    setSelectedModels(new Set());
  };

  if (modelsLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
        <span className="ml-2 text-gray-600 dark:text-gray-400">Loading models...</span>
      </div>
    );
  }

  if (modelsError) {
    return (
      <Card className="p-6">
        <div className="text-center text-red-600 dark:text-red-400">
          <p className="text-lg font-medium">Failed to load models</p>
          <p className="text-sm mt-1">Please check your connection and try again.</p>
        </div>
      </Card>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header with Statistics */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
                Model Management
              </h2>
              <p className="text-gray-600 dark:text-gray-400">
                Manage and monitor AI models
              </p>
            </div>
            <div className="flex space-x-4 text-sm">
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                  {stats.total}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Total</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                  {stats.loaded}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Loaded</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                  {stats.loading}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Loading</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-gray-600 dark:text-gray-400">
                  {stats.unloaded}
                </div>
                <div className="text-gray-500 dark:text-gray-400">Unloaded</div>
              </div>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          {/* Filters and Search */}
          <div className="flex flex-wrap gap-4 mb-6">
            <div className="flex-1 min-w-64">
              <Input
                placeholder="Search models by name or ID..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full"
              />
            </div>
            <Select
              value={statusFilter}
              onChange={(value: string) => setStatusFilter(value)}
              className="w-32"
            >
              <option value="all">All Status</option>
              <option value="loaded">Loaded</option>
              <option value="loading">Loading</option>
              <option value="unloaded">Unloaded</option>
              <option value="error">Error</option>
            </Select>
            <Select
              value={typeFilter}
              onChange={(value: string) => setTypeFilter(value)}
              className="w-32"
            >
              <option value="all">All Types</option>
              <option value="language">Language</option>
              <option value="vision">Vision</option>
              <option value="audio">Audio</option>
              <option value="multimodal">Multimodal</option>
            </Select>
          </div>

          {/* Batch Operations */}
          <ModelBatchOperations
            selectedModels={selectedModels}
            models={filteredModels}
            onClearSelection={() => setSelectedModels(new Set())}
            onBatchLoad={handleBatchLoad}
            onBatchUnload={handleBatchUnload}
            onBatchConfigure={handleBatchConfigure}
            isLoading={loadingModels.size > 0}
          />

          {/* Select All Checkbox */}
          {filteredModels.length > 0 && (
            <div className="flex items-center mb-4">
              <input
                type="checkbox"
                id="select-all"
                checked={selectedModels.size === filteredModels.length && filteredModels.length > 0}
                onChange={handleSelectAll}
                className="mr-2"
              />
              <label htmlFor="select-all" className="text-sm text-gray-600 dark:text-gray-400">
                Select all visible models
              </label>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Models Grid */}
      {filteredModels.length === 0 ? (
        <Card className="p-8">
          <div className="text-center text-gray-500 dark:text-gray-400">
            <p className="text-lg">No models found</p>
            <p className="text-sm mt-1">
              {searchTerm || statusFilter !== 'all' || typeFilter !== 'all'
                ? 'Try adjusting your filters'
                : 'No models are available'}
            </p>
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredModels.map((model: ModelInfo) => (
            <div key={model.id} className="relative">
              {/* Selection Checkbox */}
              <input
                type="checkbox"
                checked={selectedModels.has(model.id)}
                onChange={(e) => handleSelectModel(model.id, e.target.checked)}
                className="absolute top-2 left-2 z-10"
              />
              
              <ModelCard
                model={model}
                metrics={metrics?.[model.id]}
                onLoad={() => handleLoadModel(model.id)}
                onUnload={() => handleUnloadModel(model.id)}
                onConfigure={() => handleConfigureModel(model.id)}
                isLoading={loadingModels.has(model.id)}
              />
            </div>
          ))}
        </div>
      )}

      {/* Status Groups (Alternative View) */}
      {filteredModels.length > 6 && (
        <div className="space-y-6">
          {Object.entries(modelsByStatus).map(([status, statusModels]) => {
            if (statusModels.length === 0) return null;
            
            return (
              <Card key={status}>
                <CardHeader>
                  <h3 className="text-lg font-semibold capitalize text-gray-900 dark:text-white">
                    {status} Models ({statusModels.length})
                  </h3>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {statusModels.map((model: ModelInfo) => (
                      <ModelCard
                        key={model.id}
                        model={model}
                        metrics={metrics?.[model.id]}
                        onLoad={() => handleLoadModel(model.id)}
                        onUnload={() => handleUnloadModel(model.id)}
                        onConfigure={() => handleConfigureModel(model.id)}
                        isLoading={loadingModels.has(model.id)}
                      />
                    ))}
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Confirmation Dialog */}
      <ModelConfirmationDialog
        isOpen={confirmationDialog.isOpen}
        onClose={() => setConfirmationDialog({ isOpen: false, model: null, action: 'load' })}
        onConfirm={() => {
          if (confirmationDialog.model) {
            if (confirmationDialog.action !== 'delete') {
              executeModelOperation(confirmationDialog.model.id, confirmationDialog.action);
            }
          }
        }}
        model={confirmationDialog.model}
        action={confirmationDialog.action === 'delete' ? 'unload' : confirmationDialog.action}
        isLoading={confirmationDialog.model ? loadingModels.has(confirmationDialog.model.id) : false}
      />

      {/* Configuration Dialog */}
      <ModelConfigurationDialog
        isOpen={configurationDialog.isOpen}
        onClose={() => setConfigurationDialog({ isOpen: false, model: null })}
        onSave={handleSaveConfiguration}
        model={configurationDialog.model}
        isLoading={false}
      />
    </div>
  );
};