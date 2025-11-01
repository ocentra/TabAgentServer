import React, { useState } from 'react';
import { Card } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { Modal } from '../../ui/Modal';

export interface AlertRule {
  id: string;
  name: string;
  description: string;
  metric: 'cpu_usage' | 'memory_usage' | 'disk_usage' | 'error_rate' | 'response_time';
  condition: 'greater_than' | 'less_than' | 'equals';
  threshold: number;
  severity: 'info' | 'warning' | 'error';
  enabled: boolean;
  cooldown: number; // minutes
  lastTriggered?: Date;
}

interface AlertRulesProps {
  className?: string;
}

const defaultRules: AlertRule[] = [
  {
    id: 'high-cpu',
    name: 'High CPU Usage',
    description: 'Alert when CPU usage exceeds threshold',
    metric: 'cpu_usage',
    condition: 'greater_than',
    threshold: 80,
    severity: 'warning',
    enabled: true,
    cooldown: 5,
  },
  {
    id: 'critical-cpu',
    name: 'Critical CPU Usage',
    description: 'Alert when CPU usage is critically high',
    metric: 'cpu_usage',
    condition: 'greater_than',
    threshold: 95,
    severity: 'error',
    enabled: true,
    cooldown: 2,
  },
  {
    id: 'high-memory',
    name: 'High Memory Usage',
    description: 'Alert when memory usage exceeds threshold',
    metric: 'memory_usage',
    condition: 'greater_than',
    threshold: 85,
    severity: 'warning',
    enabled: true,
    cooldown: 5,
  },
  {
    id: 'low-disk',
    name: 'Low Disk Space',
    description: 'Alert when disk usage is high',
    metric: 'disk_usage',
    condition: 'greater_than',
    threshold: 90,
    severity: 'error',
    enabled: true,
    cooldown: 30,
  },
  {
    id: 'high-error-rate',
    name: 'High Error Rate',
    description: 'Alert when error rate exceeds threshold',
    metric: 'error_rate',
    condition: 'greater_than',
    threshold: 5,
    severity: 'error',
    enabled: true,
    cooldown: 10,
  },
  {
    id: 'slow-response',
    name: 'Slow Response Time',
    description: 'Alert when response time is too high',
    metric: 'response_time',
    condition: 'greater_than',
    threshold: 1000,
    severity: 'warning',
    enabled: true,
    cooldown: 15,
  },
];

export const AlertRules: React.FC<AlertRulesProps> = ({ className = '' }) => {
  const [rules, setRules] = useState<AlertRule[]>(defaultRules);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<AlertRule | null>(null);

  const handleToggleRule = (id: string) => {
    setRules(prev =>
      prev.map(rule =>
        rule.id === id ? { ...rule, enabled: !rule.enabled } : rule
      )
    );
  };

  const handleEditRule = (rule: AlertRule) => {
    setEditingRule(rule);
    setIsModalOpen(true);
  };

  const handleCreateRule = () => {
    setEditingRule({
      id: '',
      name: '',
      description: '',
      metric: 'cpu_usage',
      condition: 'greater_than',
      threshold: 80,
      severity: 'warning',
      enabled: true,
      cooldown: 5,
    });
    setIsModalOpen(true);
  };

  const handleSaveRule = (rule: AlertRule) => {
    if (rule.id) {
      // Update existing rule
      setRules(prev =>
        prev.map(r => r.id === rule.id ? rule : r)
      );
    } else {
      // Create new rule
      const newRule = {
        ...rule,
        id: Math.random().toString(36).substr(2, 9),
      };
      setRules(prev => [...prev, newRule]);
    }
    setIsModalOpen(false);
    setEditingRule(null);
  };

  const handleDeleteRule = (id: string) => {
    setRules(prev => prev.filter(rule => rule.id !== id));
  };

  const getSeverityColor = (severity: AlertRule['severity']) => {
    switch (severity) {
      case 'error':
        return 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900/20';
      case 'warning':
        return 'text-yellow-600 bg-yellow-100 dark:text-yellow-400 dark:bg-yellow-900/20';
      default:
        return 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900/20';
    }
  };

  const getMetricLabel = (metric: AlertRule['metric']) => {
    switch (metric) {
      case 'cpu_usage': return 'CPU Usage';
      case 'memory_usage': return 'Memory Usage';
      case 'disk_usage': return 'Disk Usage';
      case 'error_rate': return 'Error Rate';
      case 'response_time': return 'Response Time';
      default: return metric;
    }
  };

  const getConditionLabel = (condition: AlertRule['condition']) => {
    switch (condition) {
      case 'greater_than': return '>';
      case 'less_than': return '<';
      case 'equals': return '=';
      default: return condition;
    }
  };

  const getThresholdUnit = (metric: AlertRule['metric']) => {
    switch (metric) {
      case 'cpu_usage':
      case 'memory_usage':
      case 'disk_usage':
      case 'error_rate':
        return '%';
      case 'response_time':
        return 'ms';
      default:
        return '';
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Alert Rules
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            Configure automated alerts for system metrics
          </p>
        </div>
        <Button onClick={handleCreateRule} variant="default">
          Create Rule
        </Button>
      </div>

      {/* Rules List */}
      <Card className="p-6">
        <div className="space-y-4">
          {rules.map((rule) => (
            <div
              key={rule.id}
              className="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
            >
              <div className="flex items-center space-x-4">
                <div className="flex items-center">
                  <input
                    type="checkbox"
                    checked={rule.enabled}
                    onChange={() => handleToggleRule(rule.id)}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                </div>
                
                <div className="flex-1">
                  <div className="flex items-center space-x-2">
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      {rule.name}
                    </h3>
                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${getSeverityColor(rule.severity)}`}>
                      {rule.severity}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    {rule.description}
                  </p>
                  <div className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                    {getMetricLabel(rule.metric)} {getConditionLabel(rule.condition)} {rule.threshold}{getThresholdUnit(rule.metric)}
                    {rule.lastTriggered && (
                      <span className="ml-2">
                        • Last triggered: {rule.lastTriggered.toLocaleString()}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => handleEditRule(rule)}
                >
                  Edit
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => handleDeleteRule(rule.id)}
                  className="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-200"
                >
                  Delete
                </Button>
              </div>
            </div>
          ))}
        </div>
      </Card>

      {/* Edit/Create Rule Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
          setEditingRule(null);
        }}
        title={editingRule?.id ? 'Edit Alert Rule' : 'Create Alert Rule'}
      >
        {editingRule && (
          <AlertRuleForm
            rule={editingRule}
            onSave={handleSaveRule}
            onCancel={() => {
              setIsModalOpen(false);
              setEditingRule(null);
            }}
          />
        )}
      </Modal>
    </div>
  );
};

interface AlertRuleFormProps {
  rule: AlertRule;
  onSave: (rule: AlertRule) => void;
  onCancel: () => void;
}

const AlertRuleForm: React.FC<AlertRuleFormProps> = ({ rule, onSave, onCancel }) => {
  const [formData, setFormData] = useState(rule);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(formData);
  };

  const handleChange = (field: keyof AlertRule, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Rule Name
        </label>
        <Input
          value={formData.name}
          onChange={(e) => handleChange('name', e.target.value)}
          placeholder="Enter rule name"
          required
        />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Description
        </label>
        <Input
          value={formData.description}
          onChange={(e) => handleChange('description', e.target.value)}
          placeholder="Enter rule description"
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Metric
          </label>
          <select
            value={formData.metric}
            onChange={(e) => handleChange('metric', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
          >
            <option value="cpu_usage">CPU Usage</option>
            <option value="memory_usage">Memory Usage</option>
            <option value="disk_usage">Disk Usage</option>
            <option value="error_rate">Error Rate</option>
            <option value="response_time">Response Time</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Condition
          </label>
          <select
            value={formData.condition}
            onChange={(e) => handleChange('condition', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
          >
            <option value="greater_than">Greater than</option>
            <option value="less_than">Less than</option>
            <option value="equals">Equals</option>
          </select>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Threshold
          </label>
          <Input
            type="number"
            value={formData.threshold}
            onChange={(e) => handleChange('threshold', Number(e.target.value))}
            placeholder="Enter threshold value"
            required
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Severity
          </label>
          <select
            value={formData.severity}
            onChange={(e) => handleChange('severity', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
          >
            <option value="info">Info</option>
            <option value="warning">Warning</option>
            <option value="error">Error</option>
          </select>
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Cooldown (minutes)
        </label>
        <Input
          type="number"
          value={formData.cooldown}
          onChange={(e) => handleChange('cooldown', Number(e.target.value))}
          placeholder="Enter cooldown period in minutes"
          min="1"
          required
        />
      </div>

      <div className="flex items-center">
        <input
          type="checkbox"
          id="enabled"
          checked={formData.enabled}
          onChange={(e) => handleChange('enabled', e.target.checked)}
          className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        />
        <label htmlFor="enabled" className="ml-2 text-sm text-gray-700 dark:text-gray-300">
          Enable this rule
        </label>
      </div>

      <div className="flex justify-end space-x-3 pt-4">
        <Button type="button" variant="secondary" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" variant="default">
          {rule.id ? 'Update Rule' : 'Create Rule'}
        </Button>
      </div>
    </form>
  );
};