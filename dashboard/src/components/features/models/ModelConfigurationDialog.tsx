import React, { useState, useEffect } from 'react';
import { Modal, ModalHeader, ModalBody, ModalFooter } from '../../ui/Modal';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { Select } from '../../ui';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo } from '../../../types/models';

interface ModelConfiguration {
  max_tokens?: number;
  temperature?: number;
  top_p?: number;
  top_k?: number;
  repetition_penalty?: number;
  context_length?: number;
  batch_size?: number;
  gpu_layers?: number;
  quantization?: string;
  rope_scaling?: number;
  custom_params?: Record<string, unknown>;
}

interface ModelConfigurationDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (config: ModelConfiguration) => void;
  model: ModelInfo | null;
  isLoading?: boolean;
}

export const ModelConfigurationDialog: React.FC<ModelConfigurationDialogProps> = ({
  isOpen,
  onClose,
  onSave,
  model,
  isLoading = false,
}) => {
  const [config, setConfig] = useState<ModelConfiguration>({});
  const [activeTab, setActiveTab] = useState<'inference' | 'performance' | 'advanced'>('inference');

  // Initialize configuration when model changes
  useEffect(() => {
    if (model) {
      setConfig({
        max_tokens: 2048,
        temperature: 0.7,
        top_p: 0.9,
        top_k: 40,
        repetition_penalty: 1.1,
        context_length: 4096,
        batch_size: 1,
        gpu_layers: -1,
        quantization: model.quantization || 'none',
        rope_scaling: 1.0,
        custom_params: {},
      });
    }
  }, [model]);

  const handleSave = () => {
    onSave(config);
  };

  const updateConfig = (key: keyof ModelConfiguration, value: unknown) => {
    setConfig(prev => ({ ...prev, [key]: value }));
  };

  if (!model) return null;

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="lg">
      <ModalHeader>
        <div className="flex items-center space-x-2">
          <span className="text-2xl">⚙️</span>
          <div>
            <h2 className="text-xl font-semibold">Configure Model</h2>
            <p className="text-sm text-gray-500 dark:text-gray-400">{model.name}</p>
          </div>
        </div>
      </ModalHeader>

      <ModalBody>
        <div className="space-y-6">
          {/* Tab Navigation */}
          <div className="flex space-x-1 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg">
            {[
              { id: 'inference', label: 'Inference', icon: '🧠' },
              { id: 'performance', label: 'Performance', icon: '⚡' },
              { id: 'advanced', label: 'Advanced', icon: '🔧' },
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

          {/* Inference Settings */}
          {activeTab === 'inference' && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Max Tokens
                  </label>
                  <Input
                    type="number"
                    value={config.max_tokens || ''}
                    onChange={(e) => updateConfig('max_tokens', parseInt(e.target.value) || 0)}
                    placeholder="2048"
                    min="1"
                    max="32768"
                  />
                  <p className="text-xs text-gray-500 mt-1">Maximum number of tokens to generate</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Temperature
                  </label>
                  <Input
                    type="number"
                    value={config.temperature || ''}
                    onChange={(e) => updateConfig('temperature', parseFloat(e.target.value) || 0)}
                    placeholder="0.7"
                    min="0"
                    max="2"
                    step="0.1"
                  />
                  <p className="text-xs text-gray-500 mt-1">Controls randomness (0 = deterministic, 2 = very random)</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Top P
                  </label>
                  <Input
                    type="number"
                    value={config.top_p || ''}
                    onChange={(e) => updateConfig('top_p', parseFloat(e.target.value) || 0)}
                    placeholder="0.9"
                    min="0"
                    max="1"
                    step="0.1"
                  />
                  <p className="text-xs text-gray-500 mt-1">Nucleus sampling threshold</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Top K
                  </label>
                  <Input
                    type="number"
                    value={config.top_k || ''}
                    onChange={(e) => updateConfig('top_k', parseInt(e.target.value) || 0)}
                    placeholder="40"
                    min="1"
                    max="100"
                  />
                  <p className="text-xs text-gray-500 mt-1">Limit vocabulary to top K tokens</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Repetition Penalty
                  </label>
                  <Input
                    type="number"
                    value={config.repetition_penalty || ''}
                    onChange={(e) => updateConfig('repetition_penalty', parseFloat(e.target.value) || 0)}
                    placeholder="1.1"
                    min="0.5"
                    max="2"
                    step="0.1"
                  />
                  <p className="text-xs text-gray-500 mt-1">Penalty for repeating tokens</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Context Length
                  </label>
                  <Input
                    type="number"
                    value={config.context_length || ''}
                    onChange={(e) => updateConfig('context_length', parseInt(e.target.value) || 0)}
                    placeholder="4096"
                    min="512"
                    max="32768"
                  />
                  <p className="text-xs text-gray-500 mt-1">Maximum context window size</p>
                </div>
              </div>
            </div>
          )}

          {/* Performance Settings */}
          {activeTab === 'performance' && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Batch Size
                  </label>
                  <Input
                    type="number"
                    value={config.batch_size || ''}
                    onChange={(e) => updateConfig('batch_size', parseInt(e.target.value) || 0)}
                    placeholder="1"
                    min="1"
                    max="32"
                  />
                  <p className="text-xs text-gray-500 mt-1">Number of sequences to process in parallel</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    GPU Layers
                  </label>
                  <Input
                    type="number"
                    value={config.gpu_layers || ''}
                    onChange={(e) => updateConfig('gpu_layers', parseInt(e.target.value) || 0)}
                    placeholder="-1"
                    min="-1"
                    max="100"
                  />
                  <p className="text-xs text-gray-500 mt-1">Number of layers to offload to GPU (-1 = all)</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Quantization
                  </label>
                  <Select
                    value={config.quantization || 'none'}
                    onChange={(value: string) => updateConfig('quantization', value)}
                  >
                    <option value="none">None (FP16)</option>
                    <option value="q4_0">Q4_0 (4-bit)</option>
                    <option value="q4_1">Q4_1 (4-bit improved)</option>
                    <option value="q5_0">Q5_0 (5-bit)</option>
                    <option value="q5_1">Q5_1 (5-bit improved)</option>
                    <option value="q8_0">Q8_0 (8-bit)</option>
                  </Select>
                  <p className="text-xs text-gray-500 mt-1">Model quantization level</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    RoPE Scaling
                  </label>
                  <Input
                    type="number"
                    value={config.rope_scaling || ''}
                    onChange={(e) => updateConfig('rope_scaling', parseFloat(e.target.value) || 0)}
                    placeholder="1.0"
                    min="0.1"
                    max="10"
                    step="0.1"
                  />
                  <p className="text-xs text-gray-500 mt-1">Rotary position embedding scaling factor</p>
                </div>
              </div>
            </div>
          )}

          {/* Advanced Settings */}
          {activeTab === 'advanced' && (
            <div className="space-y-4">
              <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                <div className="flex items-center space-x-2 text-yellow-800 dark:text-yellow-200">
                  <span>⚠️</span>
                  <span className="font-medium">Advanced Settings</span>
                </div>
                <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                  These settings are for advanced users. Incorrect values may cause the model to perform poorly or fail to load.
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Custom Parameters (JSON)
                </label>
                <textarea
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
                  rows={8}
                  value={JSON.stringify(config.custom_params || {}, null, 2)}
                  onChange={(e) => {
                    try {
                      const parsed = JSON.parse(e.target.value);
                      updateConfig('custom_params', parsed);
                    } catch {
                      // Invalid JSON, ignore
                    }
                  }}
                  placeholder={`{
  "mlock": true,
  "numa": false,
  "threads": 8,
  "use_mmap": true
}`}
                />
                <p className="text-xs text-gray-500 mt-1">
                  Additional model-specific parameters in JSON format
                </p>
              </div>
            </div>
          )}

          {/* Configuration Preview */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
            <h4 className="font-medium text-gray-900 dark:text-white mb-2">Configuration Summary</h4>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Max Tokens:</span>
                <span className="font-medium">{config.max_tokens || 'Default'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Temperature:</span>
                <span className="font-medium">{config.temperature || 'Default'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">Batch Size:</span>
                <span className="font-medium">{config.batch_size || 'Default'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">GPU Layers:</span>
                <span className="font-medium">{config.gpu_layers === -1 ? 'All' : config.gpu_layers || 'Default'}</span>
              </div>
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
            onClick={handleSave}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <LoadingSpinner size="sm" className="mr-2" />
                Saving...
              </>
            ) : (
              'Save Configuration'
            )}
          </Button>
        </div>
      </ModalFooter>
    </Modal>
  );
};