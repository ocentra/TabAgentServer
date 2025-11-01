<template>
  <div class="execution-panel">
    <div class="panel-header">
      <h3>Execution</h3>
      <div class="header-actions">
        <el-button 
          size="small" 
          type="primary" 
          @click="executeWorkflow"
          :loading="isExecuting"
        >
          <el-icon><VideoPlay /></el-icon>
          Execute
        </el-button>
        <el-button 
          size="small" 
          @click="stopExecution"
          :disabled="!isExecuting"
        >
          <el-icon><VideoPause /></el-icon>
          Stop
        </el-button>
        <el-button 
          size="small" 
          @click="clearLogs"
        >
          <el-icon><Delete /></el-icon>
          Clear
        </el-button>
      </div>
    </div>

    <div class="panel-content">
      <el-tabs v-model="activeTab" type="border-card">
        <!-- Execution Logs -->
        <el-tab-pane label="Logs" name="logs">
          <div class="logs-container">
            <div 
              v-for="log in executionLogs" 
              :key="log.id"
              :class="['log-entry', `log-${log.level}`]"
            >
              <span class="log-timestamp">{{ formatTimestamp(log.timestamp) }}</span>
              <span class="log-level">{{ log.level.toUpperCase() }}</span>
              <span class="log-node" v-if="log.node_id">[{{ getNodeName(log.node_id) }}]</span>
              <span class="log-message">{{ log.message }}</span>
            </div>
            
            <div v-if="executionLogs.length === 0" class="no-logs">
              <el-icon><Document /></el-icon>
              <p>No execution logs yet. Run a workflow to see logs here.</p>
            </div>
          </div>
        </el-tab-pane>

        <!-- Execution Results -->
        <el-tab-pane label="Results" name="results">
          <div class="results-container">
            <div v-if="executionResults.length === 0" class="no-results">
              <el-icon><DataAnalysis /></el-icon>
              <p>No execution results yet. Run a workflow to see results here.</p>
            </div>
            
            <div v-else class="results-list">
              <div 
                v-for="result in executionResults" 
                :key="result.node_id"
                class="result-item"
              >
                <div class="result-header">
                  <span class="result-node">{{ getNodeName(result.node_id) }}</span>
                  <el-tag 
                    :type="getStatusTagType(result.status)"
                    size="small"
                  >
                    {{ result.status }}
                  </el-tag>
                </div>
                
                <div v-if="result.data" class="result-data">
                  <pre>{{ JSON.stringify(result.data, null, 2) }}</pre>
                </div>
                
                <div v-if="result.error" class="result-error">
                  <el-alert 
                    :title="result.error.message"
                    type="error"
                    :closable="false"
                    show-icon
                  />
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- Performance Metrics -->
        <el-tab-pane label="Performance" name="performance">
          <div class="performance-container">
            <div v-if="!performanceMetrics" class="no-metrics">
              <el-icon><TrendCharts /></el-icon>
              <p>No performance data available. Run a workflow to see metrics.</p>
            </div>
            
            <div v-else class="metrics-grid">
              <div class="metric-card">
                <h4>Total Duration</h4>
                <p class="metric-value">{{ formatDuration(performanceMetrics.total_duration_ms) }}</p>
              </div>
              
              <div class="metric-card">
                <h4>Memory Usage</h4>
                <p class="metric-value">{{ performanceMetrics.memory_usage_mb }} MB</p>
              </div>
              
              <div class="metric-card">
                <h4>CPU Usage</h4>
                <p class="metric-value">{{ performanceMetrics.cpu_usage_percent }}%</p>
              </div>
              
              <div class="metric-card">
                <h4>Nodes Executed</h4>
                <p class="metric-value">{{ Object.keys(performanceMetrics.node_execution_times).length }}</p>
              </div>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { 
  ElTabs, ElTabPane, ElButton, ElIcon, ElTag, ElAlert 
} from 'element-plus'
import { 
  VideoPlay, VideoPause, Delete, Document, DataAnalysis, TrendCharts 
} from '@element-plus/icons-vue'
import { useWorkflowStore } from '@/stores'
import type { ExecutionLog, NodeExecutionResult, ExecutionMetrics } from '@/types'

const workflowStore = useWorkflowStore()

const activeTab = ref('logs')
const isExecuting = ref(false)

// Mock execution data (will be replaced with real data from WebSocket)
const executionLogs = ref<(ExecutionLog & { id: string })[]>([])
const executionResults = ref<NodeExecutionResult[]>([])
const performanceMetrics = ref<ExecutionMetrics | null>(null)

async function executeWorkflow() {
  if (!workflowStore.currentWorkflow) return
  
  isExecuting.value = true
  
  // Clear previous results
  executionLogs.value = []
  executionResults.value = []
  performanceMetrics.value = null
  
  // Mock execution process
  addLog('info', 'Starting workflow execution...')
  
  try {
    // Simulate execution steps
    await simulateExecution()
    addLog('info', 'Workflow execution completed successfully')
  } catch (error) {
    addLog('error', `Workflow execution failed: ${error}`)
  } finally {
    isExecuting.value = false
  }
}

function stopExecution() {
  isExecuting.value = false
  addLog('warn', 'Workflow execution stopped by user')
}

function clearLogs() {
  executionLogs.value = []
  executionResults.value = []
  performanceMetrics.value = null
}

function addLog(level: 'debug' | 'info' | 'warn' | 'error', message: string, nodeId?: string) {
  executionLogs.value.push({
    id: Date.now().toString(),
    timestamp: new Date().toISOString(),
    level,
    message,
    node_id: nodeId
  })
}

async function simulateExecution() {
  if (!workflowStore.currentWorkflow) return
  
  const nodes = workflowStore.currentWorkflow.nodes
  
  for (const node of nodes) {
    addLog('info', `Executing node: ${node.data.name || node.type}`, node.id)
    
    // Simulate processing time
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // Mock result
    const result: NodeExecutionResult = {
      node_id: node.id,
      status: 'success',
      start_time: new Date().toISOString(),
      end_time: new Date().toISOString(),
      data: { output: `Result from ${node.data.name || node.type}` }
    }
    
    executionResults.value.push(result)
    addLog('info', `Node completed successfully`, node.id)
  }
  
  // Mock performance metrics
  performanceMetrics.value = {
    total_duration_ms: nodes.length * 1000,
    memory_usage_mb: 128,
    cpu_usage_percent: 45,
    node_execution_times: nodes.reduce((acc, node) => {
      acc[node.id] = 1000
      return acc
    }, {} as Record<string, number>)
  }
}

function getNodeName(nodeId: string): string {
  if (!workflowStore.currentWorkflow) return nodeId
  
  const node = workflowStore.currentWorkflow.nodes.find(n => n.id === nodeId)
  return node?.data.name || node?.type || nodeId
}

function getStatusTagType(status: string): 'success' | 'warning' | 'info' | 'primary' | 'danger' {
  const typeMap: Record<string, 'success' | 'warning' | 'info' | 'primary' | 'danger'> = {
    'success': 'success',
    'error': 'danger',
    'running': 'warning',
    'waiting': 'info'
  }
  return typeMap[status] || 'info'
}

function formatTimestamp(timestamp: string): string {
  return new Date(timestamp).toLocaleTimeString()
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  return `${(ms / 60000).toFixed(1)}m`
}
</script>

<style scoped>
.execution-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.panel-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.panel-content {
  flex: 1;
  overflow: hidden;
}

.logs-container,
.results-container,
.performance-container {
  height: 100%;
  overflow-y: auto;
  padding: 16px;
}

.log-entry {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 4px 0;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  line-height: 1.4;
}

.log-timestamp {
  color: var(--color-text-tertiary);
  white-space: nowrap;
}

.log-level {
  font-weight: 600;
  min-width: 50px;
}

.log-level.log-info { color: #3B82F6; }
.log-level.log-warn { color: #F59E0B; }
.log-level.log-error { color: #EF4444; }
.log-level.log-debug { color: #6B7280; }

.log-node {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.log-message {
  flex: 1;
  color: var(--color-text-primary);
}

.no-logs,
.no-results,
.no-metrics {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--color-text-secondary);
  text-align: center;
}

.no-logs .el-icon,
.no-results .el-icon,
.no-metrics .el-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.result-item {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  background: var(--color-background);
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.result-node {
  font-weight: 600;
  color: var(--color-text-primary);
}

.result-data pre {
  background: var(--color-background-light);
  padding: 12px;
  border-radius: 6px;
  font-size: 12px;
  overflow-x: auto;
  margin: 0;
}

.result-error {
  margin-top: 12px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.metric-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
}

.metric-card h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.metric-value {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text-primary);
}

/* CSS Variables */
:root {
  --color-background: #ffffff;
  --color-background-light: #f8f9fa;
  --color-text-primary: #2c3e50;
  --color-text-secondary: #5a6c7d;
  --color-text-tertiary: #8b9bb3;
  --color-border: #e1e5e9;
}

[data-theme="dark"] {
  --color-background: #2d2d2d;
  --color-background-light: #404040;
  --color-text-primary: #ffffff;
  --color-text-secondary: #b0b7c3;
  --color-text-tertiary: #8b9bb3;
  --color-border: #404040;
}
</style>