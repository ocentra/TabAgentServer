<template>
  <div class="node-library">
    <!-- Top Actions -->
    <div class="library-top-actions">
      <button @click="goHome" class="icon-btn" title="Home">
        🏠
      </button>
      <button @click="openSettings" class="icon-btn" title="Settings">
        ⚙️
      </button>
    </div>
    
    <div class="library-header">
      <h3>Node Library</h3>
      <el-input
        v-model="searchQuery"
        placeholder="Search nodes..."
        size="small"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <div class="library-content">
      <div class="category-section" v-for="category in filteredCategories" :key="category.name">
        <div class="category-header" @click="toggleCategory(category.name)">
          <el-icon>
            <ArrowRight v-if="!expandedCategories.includes(category.name)" />
            <ArrowDown v-else />
          </el-icon>
          <span class="category-icon">{{ category.icon }}</span>
          <span>{{ category.displayName }}</span>
          <span class="node-count">({{ category.nodes.length }})</span>
        </div>
        
        <div v-if="expandedCategories.includes(category.name)" class="category-nodes">
                <div
                  v-for="node in category.nodes"
                  :key="node.id"
                  class="node-item"
                  draggable="true"
                  @dragstart="handleDragStart(node, $event)"
                  @dragend="handleDragEnd"
                >
            <div class="node-icon" :style="{ backgroundColor: node.color }">
              <span class="icon-emoji">{{ node.icon }}</span>
            </div>
            <div class="node-info">
              <div class="node-name">{{ node.displayName }}</div>
              <div class="node-description">{{ node.description }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElInput, ElIcon } from 'element-plus'
import { Search, ArrowRight, ArrowDown } from '@element-plus/icons-vue'
import type { NodeTypeDefinition, NodeCategory } from '@/types'

import { nodeTypes as allNodeTypes, nodeCategories, type NodeType } from '@/data/nodeTypes'

const router = useRouter()
const emit = defineEmits(['open-settings'])

const searchQuery = ref('')
const expandedCategories = ref<string[]>(['ai', 'triggers'])
const nodeTypes = ref<NodeType[]>(allNodeTypes)

const categories = computed(() => {
  return nodeCategories.map(cat => ({
    name: cat.id,
    displayName: cat.name,
    icon: cat.icon,
    nodes: nodeTypes.value.filter(node => node.category === cat.id)
  })).filter(cat => cat.nodes.length > 0)
})

const filteredCategories = computed(() => {
  if (!searchQuery.value) return categories.value
  
  return categories.value.map(category => ({
    ...category,
    nodes: category.nodes.filter(node => 
      node.displayName.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      node.description.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
  })).filter(category => category.nodes.length > 0)
})

function toggleCategory(categoryName: string) {
  const index = expandedCategories.value.indexOf(categoryName)
  if (index > -1) {
    expandedCategories.value.splice(index, 1)
  } else {
    expandedCategories.value.push(categoryName)
  }
}

// Drag & Drop constants (matching n8n)
const DRAG_EVENT_DATA_KEY = 'nodeData'

function handleDragStart(node: NodeType, event: DragEvent) {
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'copy'
    event.dataTransfer.dropEffect = 'copy'
    
    // Set drag data (matching n8n pattern)
    const dragData = {
      type: node.type,
      displayName: node.displayName,
      description: node.description,
      color: node.color,
      icon: node.icon
    }
    
    event.dataTransfer.setData(DRAG_EVENT_DATA_KEY, JSON.stringify(dragData))
  }
}

const handleDragEnd = () => {
  // Cleanup after drag
}

// Navigation actions
function goHome() {
  router.push('/')
}

function openSettings() {
  emit('open-settings')
}

onMounted(() => {
  // TODO: Load node types from API
  console.log('NodeLibrary mounted')
})
</script>

<style scoped>
.node-library {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.library-top-actions {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
}

.icon-btn {
  background: transparent;
  border: none;
  font-size: 20px;
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: var(--color--background--light-1, #f5f5f5);
}

[data-theme="dark"] .icon-btn:hover {
  background: var(--color--background, #3a3a3a);
}

.library-header {
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.library-header h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.library-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.category-section {
  margin-bottom: 8px;
}

.category-header {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  font-weight: 500;
  color: var(--color-text-secondary);
  transition: background-color 0.2s;
}

.category-header:hover {
  background-color: var(--color-background-hover);
}

.category-header .el-icon {
  margin-right: 8px;
  font-size: 14px;
}

.category-icon {
  margin-right: 8px;
  font-size: 16px;
}

.node-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.category-nodes {
  padding-left: 8px;
}

.node-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  margin: 2px 8px;
  border-radius: 6px;
  cursor: grab;
  transition: all 0.2s;
}

.node-item:hover {
  background-color: var(--color-background-hover);
  transform: translateX(2px);
}

.node-item:active {
  cursor: grabbing;
}

.node-icon {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 12px;
  color: white;
  font-size: 16px;
  flex-shrink: 0;
}

.icon-emoji {
  font-size: 18px;
  line-height: 1;
}

.node-info {
  flex: 1;
  min-width: 0;
}

.node-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 2px;
}

.node-description {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* CSS Variables */
:root {
  --color-text-primary: #2c3e50;
  --color-text-secondary: #5a6c7d;
  --color-text-tertiary: #8b9bb3;
  --color-background-hover: #f1f3f4;
  --color-border: #e1e5e9;
}

[data-theme="dark"] {
  --color-text-primary: #ffffff;
  --color-text-secondary: #b0b7c3;
  --color-text-tertiary: #8b9bb3;
  --color-background-hover: #404040;
  --color-border: #404040;
}
</style>