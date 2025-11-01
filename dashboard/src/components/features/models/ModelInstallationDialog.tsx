import React, { useState } from 'react';
import { Modal, ModalHeader, ModalBody, ModalFooter } from '../../ui/Modal';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';

interface ModelInstallationDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onInstall: (config: ModelInstallConfig) => void;
  isLoading?: boolean;
}

interface ModelInstallConfig {
  source: 'huggingface' | 'local' | 'url';
  modelId: string;
  quantization?: string;
  downloadPath?: string;
  customConfig?: Record<string, unknown>;
}

export const ModelInstallationDialog: React.FC<ModelInstallationDialogProps> = ({
  isOpen,
  onClose,
  onInstall,
  isLoading = false,
}) => {
  const [config, setConfig] = useState<ModelInstallConfig>({
    source: 'huggingface',
    modelId: '',
    quantization: 'auto',
    downloadPath: '',
  });

  const [activeTab, setActiveTab] = useState<'basic' | 'advanced'>('basic');

  const handleInstall = () => {
    if (config.modelId.trim()) {
      onInstall(config);
    }
  };

  const updateConfig = (key: keyof ModelInstallConfig, value: unknown) => {
    setConfig(prev => ({ ...prev, [key]: value }));
  };

  const popularModels = [
    { id: 'microsoft/DialoGPT-medium', name: 'DialoGPT Medium', size: '1.2GB' },
    { id: 'microsoft/DialoGPT-large', name: 'DialoGPT Large', size: '2.3GB' },
    { id: 'gpt2', name: 'GPT-2', size: '548MB' },
    { id: 'distilbert-base-uncased', name: 'DistilBERT Base', size: '268MB' },
    { id: 'sentence-transformers/all-MiniLM-L6-v2', name: 'MiniLM L6 v2', size: '90MB' },
  ];

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="lg">
      <ModalHeader>
        <div className="flex items-center space-x-2">
          <span className="text-2xl">📦</span>
          <h2 className="text-xl font-semibold">Install New Model</h2>
        </div>
      </ModalHeader>

      <ModalBody>
        <div className="space-y-6">
          {/* Tab Navigation */}
          <div className="flex space-x-1 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg">
            {[
              { id: 'basic', label: 'Basic Setup', icon: '🎯' },
              { id: 'advanced', label: 'Advanced Options', icon: '⚙️' },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`flex-1 flex items-center justify-center space-x-2 px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                  activeTab === tab.id
                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                }`}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </div>

          {/* Basic Setup */}
          {activeTab === 'basic' && (
            <div className="space-y-4">
              {/* Source Selection */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Model Source
                </label>
                <Select
                  value={config.source}
                  onChange={(value: string) => updateConfig('source', value)}
                >
                  <option value="huggingface">Hugging Face Hub</option>
                  <option value="local">Local File</option>
                  <option value="url">Custom URL</option>
                </Select>
              </div>

              {/* Model ID/Path */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  {config.source === 'huggingface' ? 'Model ID' : 
                   config.source === 'local' ? 'File Path' : 'Model URL'}
                </label>
                <Input
                  value={config.modelId}
                  onChange={(e) => updateConfig('modelId', e.target.value)}
                  placeholder={
                    config.source === 'huggingface' ? 'e.g., microsoft/DialoGPT-medium' :
                    config.source === 'local' ? 'e.g., /path/to/model' :
                    'e.g., https://example.com/model.bin'
                  }
                />
              </div>

              {/* Popular Models (Hugging Face only) */}
              {config.source === 'huggingface' && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Popular Models
                  </label>
                  <div className="grid grid-cols-1 gap-2">
                    {popularModels.map((model) => (
                      <button
                        key={model.id}
                        onClick={() => updateConfig('modelId', model.id)}
                        className="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg hover:border-blue-500 dark:hover:border-blue-400 transition-colors text-left"
                      >
                        <div>
                          <div className="font-medium text-sm">{model.name}</div>
                          <div className="text-xs text-gray-500 dark:text-gray-400">{model.id}</div>
                        </div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">{model.size}</div>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Quantization */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Quantization
                </label>
                <Select
                  value={config.quantization || 'auto'}
                  onChange={(value: string) => updateConfig('quantization', value)}
                >
                  <option value="auto">Auto (Recommended)</option>
                  <option value="none">None (FP16)</option>
                  <option value="q4_0">Q4_0 (4-bit)</option>
                  <option value="q4_1">Q4_1 (4-bit improved)</option>
                  <option value="q5_0">Q5_0 (5-bit)</option>
                  <option value="q5_1">Q5_1 (5-bit improved)</option>
                  <option value="q8_0">Q8_0 (8-bit)</option>
                </Select>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Lower bit quantization reduces memory usage but may affect quality
                </p>
              </div>
            </div>
          )}

          {/* Advanced Options */}
          {activeTab === 'advanced' && (
            <div className="space-y-4">
              {/* Download Path */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Download Path (Optional)
                </label>
                <Input
                  value={config.downloadPath || ''}
                  onChange={(e) => updateConfig('downloadPath', e.target.value)}
                  placeholder="Leave empty for default location"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Custom directory to store the model files
                </p>
              </div>

              {/* Custom Configuration */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Custom Configuration (JSON)
                </label>
                <textarea
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
                  rows={6}
                  value={JSON.stringify(config.customConfig || {}, null, 2)}
                  onChange={(e) => {
                    try {
                      const parsed = JSON.parse(e.target.value);
                      updateConfig('customConfig', parsed);
                    } catch {
                      // Invalid JSON, ignore
                    }
                  }}
                  placeholder={`{
  "trust_remote_code": false,
  "use_auth_token": false,
  "revision": "main",
  "cache_dir": null
}`}
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Additional configuration options for model loading
                </p>
              </div>

              {/* Installation Notes */}
              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                <div className="flex items-start space-x-2">
                  <span className="text-blue-600 dark:text-blue-400 mt-0.5">ℹ️</span>
                  <div className="text-sm text-blue-800 dark:text-blue-200">
                    <p className="font-medium mb-1">Installation Notes:</p>
                    <ul className="space-y-1 text-xs">
                      <li>• Large models may take several minutes to download</li>
                      <li>• Ensure you have sufficient disk space available</li>
                      <li>• Some models may require authentication tokens</li>
                      <li>• Models will be automatically validated after installation</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Installation Preview */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <h4 className="font-medium text-gray-900 dark:text-white mb-2">Installation Summary</h4>
            <div className="space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Source:</span>
                <span className="font-medium capitalize">{config.source}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Model:</span>
                <span className="font-medium">{config.modelId || 'Not specified'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Quantization:</span>
                <span className="font-medium">{config.quantization || 'Auto'}</span>
              </div>
              {config.downloadPath && (
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Path:</span>
                  <span className="font-medium">{config.downloadPath}</span>
                </div>
              )}
            </div>
          </div>
        </div>
      </ModalBody>

      <ModalFooter>
        <div className="flex space-x-3">
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isLoading}
          >
            Cancel
          </Button>
          <Button
            onClick={handleInstall}
            disabled={isLoading || !config.modelId.trim()}
          >
            {isLoading ? (
              <>
                <LoadingSpinner size="sm" className="mr-2" />
                Installing...
              </>
            ) : (
              'Install Model'
            )}
          </Button>
        </div>
      </ModalFooter>
    </Modal>
  );
};