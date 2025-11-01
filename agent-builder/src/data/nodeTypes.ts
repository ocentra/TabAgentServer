// Node type definitions for the workflow builder
export interface NodeType {
  id: string
  name: string
  displayName: string
  category: string
  icon: string
  color: string
  description: string
}

export const nodeCategories = [
  { id: 'ai', name: 'AI & LLM', icon: '🤖' },
  { id: 'data', name: 'Data & Storage', icon: '💾' },
  { id: 'logic', name: 'Flow Control', icon: '⚡' },
  { id: 'communication', name: 'Communication', icon: '📧' },
  { id: 'transform', name: 'Transform', icon: '🔄' },
  { id: 'triggers', name: 'Triggers', icon: '🎯' }
]

export const nodeTypes: NodeType[] = [
  // AI & LLM
  {
    id: 'openai-gpt4',
    name: 'OpenAI GPT-4',
    displayName: 'GPT-4',
    category: 'ai',
    icon: '🤖',
    color: '#10B981',
    description: 'OpenAI GPT-4 language model'
  },
  {
    id: 'anthropic-claude',
    name: 'Anthropic Claude',
    displayName: 'Claude',
    category: 'ai',
    icon: '🧠',
    color: '#8B5CF6',
    description: 'Anthropic Claude AI assistant'
  },
  {
    id: 'local-llm',
    name: 'Local LLM',
    displayName: 'Local Model',
    category: 'ai',
    icon: '💻',
    color: '#6366F1',
    description: 'Run local language models'
  },

  // Data & Storage
  {
    id: 'database',
    name: 'Database',
    displayName: 'Database',
    category: 'data',
    icon: '💾',
    color: '#3B82F6',
    description: 'Store and retrieve data from database'
  },
  {
    id: 'google-sheets',
    name: 'Google Sheets',
    displayName: 'Google Sheets',
    category: 'data',
    icon: '📊',
    color: '#34A853',
    description: 'Read/write Google Sheets'
  },
  {
    id: 'airtable',
    name: 'Airtable',
    displayName: 'Airtable',
    category: 'data',
    icon: '🗂️',
    color: '#FCBF00',
    description: 'Interact with Airtable bases'
  },

  // Flow Control
  {
    id: 'condition',
    name: 'IF Condition',
    displayName: 'IF',
    category: 'logic',
    icon: '⚡',
    color: '#F59E0B',
    description: 'Conditional logic branching'
  },
  {
    id: 'switch',
    name: 'Switch',
    displayName: 'Switch',
    category: 'logic',
    icon: '🔀',
    color: '#F97316',
    description: 'Multiple condition routing'
  },
  {
    id: 'loop',
    name: 'Loop',
    displayName: 'Loop',
    category: 'logic',
    icon: '🔁',
    color: '#EC4899',
    description: 'Iterate over items'
  },

  // Communication
  {
    id: 'email',
    name: 'Email',
    displayName: 'Send Email',
    category: 'communication',
    icon: '📧',
    color: '#EF4444',
    description: 'Send emails via SMTP'
  },
  {
    id: 'slack',
    name: 'Slack',
    displayName: 'Slack',
    category: 'communication',
    icon: '💬',
    color: '#4A154B',
    description: 'Send Slack messages'
  },
  {
    id: 'discord',
    name: 'Discord',
    displayName: 'Discord',
    category: 'communication',
    icon: '🎮',
    color: '#5865F2',
    description: 'Send Discord messages'
  },

  // Transform
  {
    id: 'code',
    name: 'Code',
    displayName: 'Run Code',
    category: 'transform',
    icon: '💻',
    color: '#6B7280',
    description: 'Execute JavaScript/Python code'
  },
  {
    id: 'json',
    name: 'JSON',
    displayName: 'Parse JSON',
    category: 'transform',
    icon: '📝',
    color: '#14B8A6',
    description: 'Parse and manipulate JSON'
  },
  {
    id: 'transform',
    name: 'Transform',
    displayName: 'Transform Data',
    category: 'transform',
    icon: '🔄',
    color: '#06B6D4',
    description: 'Transform data structure'
  },

  // Triggers
  {
    id: 'webhook',
    name: 'Webhook',
    displayName: 'Webhook',
    category: 'triggers',
    icon: '🎯',
    color: '#EF4444',
    description: 'Trigger via HTTP webhook'
  },
  {
    id: 'schedule',
    name: 'Schedule',
    displayName: 'Schedule',
    category: 'triggers',
    icon: '⏰',
    color: '#F59E0B',
    description: 'Run on a schedule'
  },
  {
    id: 'manual',
    name: 'Manual',
    displayName: 'Manual Trigger',
    category: 'triggers',
    icon: '▶️',
    color: '#10B981',
    description: 'Manually trigger workflow'
  }
]

export function getNodesByCategory(categoryId: string): NodeType[] {
  return nodeTypes.filter(node => node.category === categoryId)
}

export function getNodeById(id: string): NodeType | undefined {
  return nodeTypes.find(node => node.id === id)
}

