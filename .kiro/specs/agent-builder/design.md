# Agent Builder Design Document

## Overview

The Agent Builder is a Vue 3-based visual workflow editor that replicates n8n's architecture and user experience while integrating with the TabAgent ecosystem. The system enables users to create complex AI agent workflows through a drag-and-drop interface, leveraging Vue Flow for canvas interactions and Pinia for state management.

## Deployment and Access Strategy

### Single Source of Truth Architecture
The Agent Builder is deployed as **one standalone Vue 3 application** served at `localhost:3000/agent-builder` by the TabAgent Rust server. This single application provides multiple access points while maintaining data consistency and real-time synchronization.

### Multiple Access Points
1. **Extension Access** - Browser extension opens `localhost:3000/agent-builder` in a new tab for full workflow editing experience
2. **Dashboard Integration** - React dashboard embeds `localhost:3000/agent-builder` via iframe at `/workflows` route for integrated viewing
3. **Direct Access** - Users can navigate directly to `localhost:3000/agent-builder` for standalone usage

### Synchronization and Data Flow
- **Single Data Source**: All access points connect to the same Rust backend APIs
- **Real-time Sync**: WebSocket connections ensure changes are reflected across all access points instantly
- **Consistent State**: Workflow modifications in dashboard iframe automatically sync with extension tab and vice versa
- **Shared Sessions**: User authentication and workflow state maintained across all access methods

### Benefits of This Architecture
- **Build Once, Use Everywhere**: Single Vue 3 codebase serves all use cases
- **No Technology Mixing**: Clean separation between React dashboard and Vue Agent Builder
- **Always Synchronized**: Real-time updates via Rust WebSocket ensure consistency
- **Flexible Access**: Users can choose their preferred interaction method
- **Easy Maintenance**: Single application to maintain and deploy

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Agent Builder (Vue 3 UI Layer)                │
├─────────────────────────────────────────────────────────────┤
│  Canvas Layer (Vue Flow) - Copy n8n's UI/UX               │
│  ├── WorkflowCanvas.vue                                    │
│  ├── Canvas.vue                                            │
│  ├── CanvasNode.vue                                        │
│  └── CanvasConnection.vue                                  │
├─────────────────────────────────────────────────────────────┤
│  UI Components (Element Plus + n8n Design)                │
│  ├── NodeLibrary.vue                                       │
│  ├── PropertiesPanel.vue                                   │
│  ├── Toolbar.vue                                           │
│  └── StatusBar.vue                                         │
├─────────────────────────────────────────────────────────────┤
│  State Management (Pinia) - UI State Only                 │
│  ├── canvasStore.ts      (viewport, selection)            │
│  ├── uiStore.ts          (panels, modals)                 │
│  └── connectionStore.ts  (WebSocket management)           │
└─────────────────────────────────────────────────────────────┘
                              │
                    (gRPC/REST + WebSocket)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              TabAgent Rust Workflow Engine                 │
├─────────────────────────────────────────────────────────────┤
│  Workflow Management                                       │
│  ├── Workflow CRUD & Validation                           │
│  ├── Node Type Registry                                    │
│  ├── Workflow Execution Engine                            │
│  └── Real-time Execution Updates                          │
├─────────────────────────────────────────────────────────────┤
│  Node Execution System                                     │
│  ├── AI Model Nodes (GPT, Claude, Local)                  │
│  ├── Data Connector Nodes (Google, Email, Asana)          │
│  ├── Logic Nodes (Conditions, Loops, Transform)           │
│  └── Trigger Nodes (Webhooks, Schedules)                  │
├─────────────────────────────────────────────────────────────┤
│  Integration Layer                                         │
│  ├── Existing TabAgent Models & Database                  │
│  ├── External API Connectors                              │
│  ├── Credential Management                                 │
│  └── Execution History & Logging                          │
└─────────────────────────────────────────────────────────────┘
```

### Technology Stack

**Core Framework:**
- Vue 3.4+ with Composition API
- TypeScript 5.2+
- Vite 5.0+ for build tooling

**Canvas and Visualization:**
- @vue-flow/core 1.45.0 (exact version from n8n)
- @vue-flow/background, @vue-flow/controls, @vue-flow/minimap
- @dagrejs/dagre for auto-layout

**UI Components:**
- Element Plus (matching n8n's UI library)
- Custom components copied from n8n's design system
- Lucide Vue for icons

**State Management:**
- Pinia for global state
- VueUse composables for reactive utilities

**Code Editor:**
- CodeMirror 6 for node configuration
- Multiple language support (JavaScript, Python, JSON, SQL)

## Components and Interfaces

### Core Canvas Components

#### WorkflowCanvas.vue
```typescript
interface WorkflowCanvasProps {
  workflowId?: string;
  workflow: WorkflowData;
  readOnly?: boolean;
  executing?: boolean;
}

interface WorkflowCanvasEmits {
  'workflow:save': (workflow: WorkflowData) => void;
  'workflow:execute': (workflowId: string) => void;
  'node:select': (nodeId: string) => void;
}
```

#### Canvas.vue
```typescript
interface CanvasProps {
  nodes: CanvasNode[];
  connections: CanvasConnection[];
  viewport: ViewportTransform;
}

interface CanvasEmits {
  'nodes:move': (event: NodeMoveEvent[]) => void;
  'connection:create': (connection: ConnectionCreateData) => void;
  'connection:delete': (connectionId: string) => void;
}
```

#### CanvasNode.vue
```typescript
interface CanvasNodeProps {
  id: string;
  data: CanvasNodeData;
  selected: boolean;
  dragging: boolean;
}

interface CanvasNodeData {
  type: string;
  name: string;
  parameters: Record<string, any>;
  position: { x: number; y: number };
  inputs: NodeInput[];
  outputs: NodeOutput[];
  status?: 'idle' | 'running' | 'success' | 'error';
}
```

### Node Type System

#### Node Categories
```typescript
enum NodeCategory {
  AI_MODELS = 'ai-models',
  DATA_CONNECTORS = 'data-connectors', 
  LOGIC = 'logic',
  TRIGGERS = 'triggers',
  ACTIONS = 'actions',
  UTILITIES = 'utilities'
}

interface NodeTypeDefinition {
  name: string;
  displayName: string;
  category: NodeCategory;
  description: string;
  icon: string;
  color: string;
  inputs: NodeInputDefinition[];
  outputs: NodeOutputDefinition[];
  parameters: NodeParameterDefinition[];
  credentials?: CredentialDefinition[];
}
```

#### AI Model Nodes
```typescript
interface AIModelNode extends NodeTypeDefinition {
  category: NodeCategory.AI_MODELS;
  modelProvider: 'openai' | 'anthropic' | 'local' | 'tabagent';
  supportedModels: string[];
  maxTokens?: number;
  streaming?: boolean;
}
```

#### Data Connector Nodes
```typescript
interface DataConnectorNode extends NodeTypeDefinition {
  category: NodeCategory.DATA_CONNECTORS;
  service: 'google' | 'email' | 'asana' | 'slack' | 'notion';
  authType: 'oauth2' | 'api-key' | 'basic';
  operations: ConnectorOperation[];
}
```

### State Management Architecture

#### Workflow Store
```typescript
interface WorkflowState {
  currentWorkflow: WorkflowData | null;
  workflows: WorkflowData[];
  activeNodeId: string | null;
  selectedNodeIds: string[];
  clipboard: ClipboardData | null;
  isDirty: boolean;
}

interface WorkflowActions {
  loadWorkflow(id: string): Promise<void>;
  saveWorkflow(): Promise<void>;
  createNode(type: string, position: XYPosition): void;
  deleteNode(id: string): void;
  updateNodeData(id: string, data: Partial<CanvasNodeData>): void;
  createConnection(source: string, target: string): void;
  deleteConnection(id: string): void;
}
```

#### Canvas Store
```typescript
interface CanvasState {
  viewport: ViewportTransform;
  zoom: number;
  pannable: boolean;
  selectable: boolean;
  connecting: boolean;
  draggedNodeType: string | null;
}

interface CanvasActions {
  setViewport(viewport: ViewportTransform): void;
  fitView(): void;
  zoomIn(): void;
  zoomOut(): void;
  resetZoom(): void;
}
```

#### Connection Store (WebSocket Management)
```typescript
interface ConnectionState {
  isConnected: boolean;
  executionUpdates: Map<string, ExecutionUpdate>;
  subscriptions: Set<string>;
  reconnectAttempts: number;
}

interface ConnectionActions {
  connect(): Promise<void>;
  disconnect(): void;
  subscribeToExecution(executionId: string): void;
  unsubscribeFromExecution(executionId: string): void;
  sendCommand(command: WorkflowCommand): void;
}
```

## Data Models

### Workflow Data Model
```typescript
interface WorkflowData {
  id: string;
  name: string;
  description?: string;
  nodes: CanvasNode[];
  connections: CanvasConnection[];
  settings: WorkflowSettings;
  createdAt: Date;
  updatedAt: Date;
  version: number;
}

interface WorkflowSettings {
  timezone: string;
  saveDataErrorExecution: 'all' | 'none';
  saveDataSuccessExecution: 'all' | 'none';
  saveManualExecutions: boolean;
  callerPolicy: 'workflowsFromSameOwner' | 'workflowsFromAList' | 'any';
}
```

### Node Data Model
```typescript
interface CanvasNode {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: CanvasNodeData;
  selected?: boolean;
  dragging?: boolean;
  dimensions?: { width: number; height: number };
}

interface CanvasConnection {
  id: string;
  source: string;
  target: string;
  sourceHandle: string;
  targetHandle: string;
  type?: 'default' | 'smoothstep' | 'step';
  animated?: boolean;
  style?: Record<string, any>;
}
```

### Execution Data Model
```typescript
interface ExecutionData {
  id: string;
  workflowId: string;
  status: 'new' | 'running' | 'success' | 'error' | 'canceled';
  startedAt: Date;
  finishedAt?: Date;
  mode: 'manual' | 'trigger' | 'webhook';
  data: ExecutionNodeData[];
}

interface ExecutionNodeData {
  nodeId: string;
  status: 'waiting' | 'running' | 'success' | 'error';
  startTime?: Date;
  endTime?: Date;
  data?: any[];
  error?: ExecutionError;
}
```

## Error Handling

### Error Types
```typescript
interface WorkflowError {
  type: 'validation' | 'execution' | 'network' | 'permission';
  message: string;
  nodeId?: string;
  details?: Record<string, any>;
}

interface ValidationError extends WorkflowError {
  type: 'validation';
  field: string;
  constraint: string;
}

interface ExecutionError extends WorkflowError {
  type: 'execution';
  nodeId: string;
  step: number;
  stack?: string;
}
```

### Error Handling Strategy
1. **Validation Errors**: Show inline validation messages on nodes and parameters
2. **Execution Errors**: Display error states on nodes with detailed error information
3. **Network Errors**: Show toast notifications with retry options
4. **Permission Errors**: Redirect to authentication or show permission denied messages

## Testing Strategy

### Unit Testing
- Vue Test Utils for component testing
- Vitest for test runner (matching n8n's setup)
- Mock stores and API calls
- Test node rendering, connections, and interactions

### Integration Testing
- Test workflow save/load functionality
- Test execution flow with mocked backend
- Test canvas interactions and state updates
- Test WebSocket connections for real-time updates

### E2E Testing
- Playwright for end-to-end testing
- Test complete workflow creation and execution
- Test integration with TabAgent server
- Test extension integration scenarios

### Component Testing Structure
```typescript
// Example test structure
describe('CanvasNode.vue', () => {
  it('renders node with correct type and data', () => {});
  it('handles node selection', () => {});
  it('displays execution status correctly', () => {});
  it('shows validation errors', () => {});
});

describe('WorkflowCanvas.vue', () => {
  it('creates connections between nodes', () => {});
  it('handles node drag and drop', () => {});
  it('saves workflow data', () => {});
  it('executes workflow', () => {});
});
```

## Integration Points

### TabAgent Rust Server APIs

#### Workflow Management (Rust-Powered)
```typescript
// GET /v1/agent-builder/workflows
interface GetWorkflowsResponse {
  workflows: WorkflowSummary[];
  total: number;
  page: number;
  limit: number;
}

// POST /v1/agent-builder/workflows
interface CreateWorkflowRequest {
  name: string;
  description?: string;
  nodes: WorkflowNode[];
  connections: WorkflowConnection[];
}

// PUT /v1/agent-builder/workflows/:id
interface UpdateWorkflowRequest {
  name?: string;
  description?: string;
  nodes?: WorkflowNode[];
  connections?: WorkflowConnection[];
}

// POST /v1/agent-builder/workflows/:id/validate
interface ValidateWorkflowResponse {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}
```

#### Node Types (Rust Registry)
```typescript
// GET /v1/agent-builder/node-types
interface GetNodeTypesResponse {
  node_types: RustNodeTypeDefinition[];
  categories: NodeCategory[];
  available_models: string[];
  available_connectors: ConnectorInfo[];
}

// GET /v1/agent-builder/node-types/:type
interface GetNodeTypeResponse {
  node_type: RustNodeTypeDefinition;
  documentation: string;
  examples: NodeExample[];
  parameter_schema: JsonSchema;
}

// GET /v1/agent-builder/connectors
interface GetConnectorsResponse {
  connectors: AvailableConnector[];
  credentials_required: CredentialRequirement[];
}

interface RustNodeTypeDefinition {
  id: string;
  name: string;
  category: NodeCategory;
  description: string;
  icon: string;
  color: string;
  inputs: NodeInputDefinition[];
  outputs: NodeOutputDefinition[];
  parameters: RustParameterDefinition[];
  rust_implementation: string; // Rust struct/trait name
  execution_timeout?: number;
  resource_requirements?: ResourceRequirements;
}
```

#### Workflow Execution (Rust Engine)
```typescript
// POST /v1/agent-builder/workflows/:id/execute
interface ExecuteWorkflowRequest {
  mode: 'manual' | 'trigger' | 'test';
  input_data?: Record<string, any>;
  debug_mode?: boolean;
}

interface ExecuteWorkflowResponse {
  execution_id: string;
  status: 'queued' | 'running';
  estimated_duration?: number;
}

// GET /v1/agent-builder/executions/:id
interface GetExecutionResponse {
  execution: RustExecutionData;
  node_results: NodeExecutionResult[];
  logs: ExecutionLog[];
  performance_metrics: ExecutionMetrics;
}

// POST /v1/agent-builder/executions/:id/stop
interface StopExecutionResponse {
  stopped: boolean;
  final_status: 'cancelled' | 'error';
}

// WebSocket /ws/agent-builder/executions
interface RustExecutionUpdate {
  execution_id: string;
  node_id: string;
  status: 'waiting' | 'running' | 'success' | 'error' | 'skipped';
  progress?: number;
  data?: any;
  error?: RustExecutionError;
  timestamp: string;
}
```

### Extension Integration
```typescript
// Extension opens Agent Builder in new tab
interface ExtensionAgentBuilderAction {
  action: 'open-agent-builder';
  workflowId?: string;
  mode?: 'create' | 'edit' | 'view';
}

// Extension implementation
function openAgentBuilder(workflowId?: string) {
  const url = workflowId 
    ? `http://localhost:3000/agent-builder?workflow=${workflowId}`
    : 'http://localhost:3000/agent-builder';
  
  chrome.tabs.create({ url });
}
```

### Dashboard Integration (React)
```typescript
// Dashboard route for embedded Agent Builder
// File: dashboard/src/pages/Workflows.tsx
import React from 'react';

const WorkflowsPage: React.FC = () => {
  return (
    <div className="h-full w-full">
      <iframe
        src="http://localhost:3000/agent-builder"
        className="w-full h-full border-0"
        title="Agent Builder"
        allow="clipboard-read; clipboard-write"
      />
    </div>
  );
};

// Dashboard router configuration
const routes = [
  {
    path: '/workflows',
    element: <WorkflowsPage />,
    meta: { title: 'Agent Builder' }
  }
];
```

### Agent Builder Routing (Vue)
```typescript
// Agent Builder handles its own routing internally
// File: agent-builder/src/router.ts
const routes = [
  {
    path: '/',
    component: WorkflowEditor,
    props: (route) => ({ 
      workflowId: route.query.workflow 
    })
  },
  {
    path: '/workflow/:id',
    component: WorkflowEditor,
    props: true
  }
];
```

### Cross-Frame Communication
```typescript
// Agent Builder posts messages to parent (dashboard iframe)
interface AgentBuilderMessage {
  type: 'workflow-saved' | 'workflow-executed' | 'resize-request';
  data: any;
}

// Agent Builder sends updates to dashboard
window.parent.postMessage({
  type: 'workflow-saved',
  data: { workflowId: '123', name: 'My Workflow' }
}, 'http://localhost:3000');

// Dashboard listens for Agent Builder updates
window.addEventListener('message', (event) => {
  if (event.origin === 'http://localhost:3000') {
    handleAgentBuilderMessage(event.data);
  }
});
```

This design document provides a comprehensive blueprint for building the Agent Builder by copying n8n's proven architecture while integrating seamlessly with the TabAgent ecosystem.