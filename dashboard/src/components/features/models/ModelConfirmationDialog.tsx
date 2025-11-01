import React from 'react';
import { Modal, ModalHeader, ModalBody, ModalFooter } from '../../ui/Modal';
import { Button } from '../../ui/Button';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import type { ModelInfo } from '../../../types/models';

interface ModelConfirmationDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  model: ModelInfo | null;
  action: 'load' | 'unload' | 'delete';
  isLoading?: boolean;
}

export const ModelConfirmationDialog: React.FC<ModelConfirmationDialogProps> = ({
  isOpen,
  onClose,
  onConfirm,
  model,
  action,
  isLoading = false,
}) => {
  if (!model) return null;

  const getActionConfig = () => {
    switch (action) {
      case 'load':
        return {
          title: 'Load Model',
          message: `Are you sure you want to load "${model.name}"?`,
          details: [
            `This will allocate approximately ${model.memory_usage ? `${(model.memory_usage / 1024 / 1024 / 1024).toFixed(1)} GB` : 'unknown amount'} of memory.`,
            'The model will be available for inference after loading completes.',
          ],
          confirmText: 'Load Model',
          confirmVariant: 'default' as const,
          icon: '🚀',
        };
      case 'unload':
        return {
          title: 'Unload Model',
          message: `Are you sure you want to unload "${model.name}"?`,
          details: [
            'This will free up the memory used by the model.',
            'The model will no longer be available for inference until loaded again.',
            'Any ongoing inference requests will be cancelled.',
          ],
          confirmText: 'Unload Model',
          confirmVariant: 'outline' as const,
          icon: '📤',
        };
      case 'delete':
        return {
          title: 'Delete Model',
          message: `Are you sure you want to delete "${model.name}"?`,
          details: [
            'This action cannot be undone.',
            'The model files will be permanently removed from your system.',
            'You will need to download the model again to use it.',
          ],
          confirmText: 'Delete Model',
          confirmVariant: 'destructive' as const,
          icon: '🗑️',
        };
      default:
        return {
          title: 'Confirm Action',
          message: 'Are you sure you want to proceed?',
          details: [],
          confirmText: 'Confirm',
          confirmVariant: 'default' as const,
          icon: '❓',
        };
    }
  };

  const config = getActionConfig();

  return (
    <Modal isOpen={isOpen} onClose={onClose}>
      <ModalHeader>
        <div className="flex items-center space-x-2">
          <span className="text-2xl">{config.icon}</span>
          <h2 className="text-xl font-semibold">{config.title}</h2>
        </div>
      </ModalHeader>

      <ModalBody>
        <div className="space-y-4">
          <p className="text-gray-700 dark:text-gray-300">
            {config.message}
          </p>

          {config.details.length > 0 && (
            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
              <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
                {config.details.map((detail, index) => (
                  <li key={index} className="flex items-start space-x-2">
                    <span className="text-gray-400 mt-1">•</span>
                    <span>{detail}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Model Information */}
          <div className="border rounded-lg p-4 bg-white dark:bg-gray-900">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-gray-500 dark:text-gray-400">Type:</span>
                <p className="font-medium capitalize">{model.type}</p>
              </div>
              <div>
                <span className="text-gray-500 dark:text-gray-400">Parameters:</span>
                <p className="font-medium">
                  {model.parameters ? `${(model.parameters / 1e9).toFixed(1)}B` : 'Unknown'}
                </p>
              </div>
              {model.memory_usage && (
                <div>
                  <span className="text-gray-500 dark:text-gray-400">Memory:</span>
                  <p className="font-medium">
                    {(model.memory_usage / 1024 / 1024 / 1024).toFixed(1)} GB
                  </p>
                </div>
              )}
              {model.quantization && (
                <div>
                  <span className="text-gray-500 dark:text-gray-400">Quantization:</span>
                  <p className="font-medium">{model.quantization}</p>
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
            variant={config.confirmVariant}
            onClick={onConfirm}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <LoadingSpinner size="sm" className="mr-2" />
                {action === 'load' ? 'Loading...' : action === 'unload' ? 'Unloading...' : 'Deleting...'}
              </>
            ) : (
              config.confirmText
            )}
          </Button>
        </div>
      </ModalFooter>
    </Modal>
  );
};