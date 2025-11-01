<template>
  <div class="workflow-list">
    <header class="list-header">
      <h1>Workflows</h1>
      <el-button 
        type="primary" 
        @click="createNewWorkflow"
      >
        <el-icon><Plus /></el-icon>
        New Workflow
      </el-button>
    </header>

    <div class="list-content">
      <el-table 
        :data="workflows" 
        v-loading="loading"
        @row-click="openWorkflow"
      >
        <el-table-column prop="name" label="Name" />
        <el-table-column prop="description" label="Description" />
        <el-table-column prop="nodeCount" label="Nodes" width="100" />
        <el-table-column prop="status" label="Status" width="120">
          <template #default="{ row }">
            <el-tag 
              :type="row.status === 'active' ? 'success' : 'info'"
            >
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="updatedAt" label="Updated" width="180">
          <template #default="{ row }">
            {{ formatDate(row.updatedAt) }}
          </template>
        </el-table-column>
        <el-table-column label="Actions" width="150">
          <template #default="{ row }">
            <el-button 
              size="small" 
              @click.stop="editWorkflow(row.id)"
            >
              Edit
            </el-button>
            <el-button 
              size="small" 
              type="danger" 
              @click.stop="deleteWorkflow(row.id)"
            >
              Delete
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElButton, ElTable, ElTableColumn, ElTag, ElIcon, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import { useUIStore } from '@/stores'
import type { WorkflowSummary } from '@/types'

const router = useRouter()
const uiStore = useUIStore()

const workflows = ref<WorkflowSummary[]>([])
const loading = ref(false)

onMounted(() => {
  loadWorkflows()
})

async function loadWorkflows() {
  try {
    loading.value = true
    // TODO: Implement API call to load workflows
    // Mock data for now
    workflows.value = [
      {
        id: '1',
        name: 'Customer Support Bot',
        description: 'Automated customer support workflow',
        createdAt: '2024-01-15T10:00:00Z',
        updatedAt: '2024-01-20T14:30:00Z',
        status: 'active',
        nodeCount: 8
      },
      {
        id: '2',
        name: 'Data Processing Pipeline',
        description: 'Process and analyze customer data',
        createdAt: '2024-01-10T09:00:00Z',
        updatedAt: '2024-01-18T16:45:00Z',
        status: 'inactive',
        nodeCount: 12
      }
    ]
  } catch (error) {
    console.error('Failed to load workflows:', error)
    uiStore.addNotification({
      type: 'error',
      message: 'Failed to load workflows'
    })
  } finally {
    loading.value = false
  }
}

function createNewWorkflow() {
  router.push('/workflow/new')
}

function openWorkflow(workflow: WorkflowSummary) {
  router.push(`/workflow/${workflow.id}`)
}

function editWorkflow(id: string) {
  router.push(`/workflow/${id}`)
}

async function deleteWorkflow(id: string) {
  try {
    await ElMessageBox.confirm(
      'This will permanently delete the workflow. Continue?',
      'Warning',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
    )
    
    // TODO: Implement API call to delete workflow
    console.log('Deleting workflow:', id)
    
    // Remove from local list
    workflows.value = workflows.value.filter(w => w.id !== id)
    
    uiStore.addNotification({
      type: 'success',
      message: 'Workflow deleted successfully'
    })
  } catch (error) {
    // User cancelled or error occurred
    console.log('Delete cancelled or failed:', error)
  }
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<style scoped>
.workflow-list {
  padding: 24px;
  height: 100vh;
  background: var(--color-background);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.list-header h1 {
  margin: 0;
  font-size: 28px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.list-content {
  background: var(--color-background-light);
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

/* CSS Variables */
:root {
  --color-background: #ffffff;
  --color-background-light: #f8f9fa;
  --color-text-primary: #2c3e50;
}

[data-theme="dark"] {
  --color-background: #1a1a1a;
  --color-background-light: #2d2d2d;
  --color-text-primary: #ffffff;
}
</style>