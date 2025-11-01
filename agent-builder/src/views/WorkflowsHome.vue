<template>
  <div class="workflows-home">
    <!-- Top Bar -->
    <div class="top-bar">
      <div class="search-container">
        <el-input
          v-model="searchQuery"
          placeholder="What would you like to automate?"
          class="search-input"
          clearable
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
      </div>
      <el-button type="primary" size="large" @click="createNewWorkflow">
        <el-icon><Plus /></el-icon>
        Create Workflow
      </el-button>
    </div>

    <!-- Stats Cards -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-label">Total Workflows</div>
        <div class="stat-value">{{ stats.totalWorkflows }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Active</div>
        <div class="stat-value">{{ stats.active }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Executions (7d)</div>
        <div class="stat-value">{{ stats.executions }}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Success Rate</div>
        <div class="stat-value">{{ stats.successRate }}%</div>
      </div>
    </div>

    <!-- Workflow Suggestions -->
    <div class="suggestions-section">
      <h2>Workflow Suggestions</h2>
      <div class="suggestions-grid">
        <div
          v-for="suggestion in filteredSuggestions"
          :key="suggestion.id"
          class="suggestion-card"
          @click="useSuggestion(suggestion)"
        >
          <div class="suggestion-icon">{{ suggestion.icon }}</div>
          <h3>{{ suggestion.title }}</h3>
          <p>{{ suggestion.description }}</p>
          <div class="suggestion-tags">
            <span v-for="tag in suggestion.tags" :key="tag" class="tag">
              {{ tag }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Workflows -->
    <div class="recent-section" v-if="recentWorkflows.length > 0">
      <h2>Recent Workflows</h2>
      <div class="workflows-list">
        <div
          v-for="workflow in recentWorkflows"
          :key="workflow.id"
          class="workflow-item"
          @click="openWorkflow(workflow.id)"
        >
          <div class="workflow-icon">
            <el-icon><Document /></el-icon>
          </div>
          <div class="workflow-info">
            <h3>{{ workflow.name }}</h3>
            <p>{{ workflow.description }}</p>
          </div>
          <div class="workflow-meta">
            <span class="updated">Updated {{ formatDate(workflow.updatedAt) }}</span>
            <el-tag :type="workflow.active ? 'success' : ''">
              {{ workflow.active ? 'Active' : 'Inactive' }}
            </el-tag>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Plus, Document } from '@element-plus/icons-vue'

const router = useRouter()
const searchQuery = ref('')

// Stats
const stats = ref({
  totalWorkflows: 0,
  active: 0,
  executions: 0,
  successRate: 0
})

// Workflow suggestions
const suggestions = ref([
  {
    id: 1,
    icon: '📧',
    title: 'Summarize emails with AI',
    description: 'Automatically summarize important emails using AI',
    tags: ['AI', 'Email', 'Productivity']
  },
  {
    id: 2,
    icon: '🤖',
    title: 'Multi-agent research workflow',
    description: 'Create autonomous research agents that collaborate',
    tags: ['AI', 'Research', 'Agents']
  },
  {
    id: 3,
    icon: '📊',
    title: 'Data processing pipeline',
    description: 'Transform and analyze data automatically',
    tags: ['Data', 'ETL', 'Analytics']
  },
  {
    id: 4,
    icon: '💬',
    title: 'Customer support chatbot',
    description: 'Build an AI chatbot for customer support',
    tags: ['AI', 'Support', 'Chat']
  },
  {
    id: 5,
    icon: '📱',
    title: 'Social media automation',
    description: 'Schedule and post content across platforms',
    tags: ['Social', 'Marketing', 'Automation']
  },
  {
    id: 6,
    icon: '🎯',
    title: 'Lead qualification workflow',
    description: 'Automatically qualify and route leads',
    tags: ['Sales', 'CRM', 'Automation']
  },
  {
    id: 7,
    icon: '📝',
    title: 'Invoice processing',
    description: 'Extract and process invoice data automatically',
    tags: ['Finance', 'OCR', 'Automation']
  },
  {
    id: 8,
    icon: '🔔',
    title: 'Daily AI news digest',
    description: 'Get personalized news summaries every morning',
    tags: ['AI', 'News', 'Productivity']
  }
])

// Recent workflows
const recentWorkflows = ref([
  // Will be populated from store later
])

const filteredSuggestions = computed(() => {
  if (!searchQuery.value) return suggestions.value
  
  const query = searchQuery.value.toLowerCase()
  return suggestions.value.filter(s => 
    s.title.toLowerCase().includes(query) ||
    s.description.toLowerCase().includes(query) ||
    s.tags.some(tag => tag.toLowerCase().includes(query))
  )
})

const createNewWorkflow = () => {
  router.push('/workflow/new')
}

const useSuggestion = (suggestion: any) => {
  // Create workflow from suggestion
  router.push(`/workflow/new?template=${suggestion.id}`)
}

const openWorkflow = (id: string) => {
  router.push(`/workflow/${id}`)
}

const formatDate = (date: Date) => {
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  
  if (days === 0) return 'today'
  if (days === 1) return 'yesterday'
  if (days < 7) return `${days} days ago`
  return date.toLocaleDateString()
}
</script>

<style scoped>
.workflows-home {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
  background: var(--color-background);
  min-height: 100vh;
}

.top-bar {
  display: flex;
  gap: 16px;
  margin-bottom: 32px;
  align-items: center;
}

.search-container {
  flex: 1;
  max-width: 600px;
}

.search-input {
  width: 100%;
}

.search-input :deep(.el-input__wrapper) {
  border-radius: 8px;
  padding: 8px 16px;
  box-shadow: var(--shadow);
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 32px;
}

.stat-card {
  background: var(--color-background-light);
  padding: 20px;
  border-radius: 8px;
  box-shadow: var(--shadow);
  border: 1px solid var(--color-border);
}

.stat-label {
  font-size: 14px;
  color: var(--color--text--tint-1);
  margin-bottom: 8px;
}

.stat-value {
  font-size: 32px;
  font-weight: 600;
  color: var(--color--text--shade-1);
}

/* Suggestions Section */
.suggestions-section,
.recent-section {
  margin-bottom: 48px;
}

.suggestions-section h2,
.recent-section h2 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 20px;
  color: var(--color--text--shade-1);
}

.suggestions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.suggestion-card {
  background: var(--color-background-light);
  padding: 24px;
  border-radius: 8px;
  box-shadow: var(--shadow);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all 0.2s ease;
}

.suggestion-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow--dark);
  border-color: #ff6d5a;
}

.suggestion-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.suggestion-card h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--color--text--shade-1);
}

.suggestion-card p {
  font-size: 14px;
  color: var(--color--text--tint-1);
  margin-bottom: 12px;
  line-height: 1.5;
}

.suggestion-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.tag {
  font-size: 12px;
  padding: 4px 8px;
  background: var(--color--foreground);
  color: var(--color--text--tint-2);
  border-radius: 4px;
}

/* Recent Workflows */
.workflows-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.workflow-item {
  background: var(--color-background-light);
  padding: 20px;
  border-radius: 8px;
  box-shadow: var(--shadow);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 16px;
}

.workflow-item:hover {
  transform: translateX(4px);
  box-shadow: var(--shadow--dark);
  border-color: #ff6d5a;
}

.workflow-icon {
  font-size: 24px;
  color: var(--color--text--tint-1);
}

.workflow-info {
  flex: 1;
}

.workflow-info h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
  color: var(--color--text--shade-1);
}

.workflow-info p {
  font-size: 14px;
  color: var(--color--text--tint-1);
}

.workflow-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.updated {
  font-size: 12px;
  color: var(--color--text--tint-2);
}

/* Dark theme */
[data-theme="dark"] {
  .stat-card,
  .suggestion-card,
  .workflow-item {
    background: var(--color-background-light);
    border-color: var(--color--foreground--shade-1);
  }
  
  .tag {
    background: var(--color--foreground--shade-1);
    color: var(--color--text);
  }
}
</style>

