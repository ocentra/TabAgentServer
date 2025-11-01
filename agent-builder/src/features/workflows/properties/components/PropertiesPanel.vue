<template>
  <div class="properties-panel-content">
    <!-- Settings View -->
    <div v-if="type === 'settings'" class="settings-view">
      <div class="panel-header">
        <h3>Settings</h3>
      </div>
      
      <div class="panel-body">
        <div class="setting-item">
          <label>Theme</label>
          <button @click="toggleTheme" class="theme-toggle-btn">
            {{ isDark ? '☀️ Light Mode' : '🌙 Dark Mode' }}
          </button>
        </div>
        
        <!-- More settings will go here -->
        <div class="setting-item">
          <label>Auto Save</label>
          <label class="switch">
            <input type="checkbox" v-model="autoSave">
            <span class="slider"></span>
          </label>
        </div>
      </div>
    </div>
    
    <!-- Node Properties View -->
    <div v-else-if="type === 'node' && selectedNode" class="node-view">
      <div class="panel-header">
        <h3>{{ selectedNode.data?.label || 'Node' }} Properties</h3>
      </div>
      
      <div class="panel-body">
        <div class="property-group">
          <label>Node Name</label>
          <input 
            v-model="selectedNode.data.label" 
            class="property-input"
            placeholder="Node Name"
          />
        </div>
        
        <div class="property-group">
          <label>Node Type</label>
          <input 
            :value="selectedNode.data.type" 
            class="property-input"
            disabled
          />
        </div>
        
        <div class="property-group">
          <label>Status</label>
          <select v-model="selectedNode.data.status" class="property-select">
            <option value="idle">Idle</option>
            <option value="running">Running</option>
            <option value="success">Success</option>
            <option value="error">Error</option>
            <option value="warning">Warning</option>
          </select>
        </div>
        
        <!-- More node properties will go here -->
      </div>
    </div>
    
    <!-- Default/Empty View -->
    <div v-else class="empty-view">
      <div class="empty-state">
        <p>Select a node to view its properties</p>
        <p class="hint">or</p>
        <p>Click ⚙️ Settings to configure the app</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Props {
  type?: 'settings' | 'node' | null
  selectedNode?: any
}

defineProps<Props>()

const isDark = ref(true)
const autoSave = ref(false)

onMounted(() => {
  const currentTheme = document.documentElement.getAttribute('data-theme')
  isDark.value = currentTheme === 'dark'
})

const toggleTheme = () => {
  isDark.value = !isDark.value
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}
</script>

<style scoped>
.properties-panel-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  color: var(--color--text--shade-1, #333);
}

[data-theme="dark"] .properties-panel-content {
  color: var(--color--text--shade-1, #e0e0e0);
}

.panel-header {
  padding: 20px;
  border-bottom: 1px solid var(--color--foreground--shade-2, #ddd);
}

[data-theme="dark"] .panel-header {
  border-bottom-color: var(--color--foreground--shade-1, #444);
}

.panel-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.panel-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

/* Settings View */
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
}

.setting-item label {
  font-weight: 500;
}

.theme-toggle-btn {
  padding: 8px 16px;
  background: var(--color--background--light-2, white);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  box-shadow: var(--shadow, 0 2px 4px rgba(0, 0, 0, 0.1));
}

.theme-toggle-btn:hover {
  background: var(--color--background--light-1, #f5f5f5);
  box-shadow: var(--shadow--dark, 0 4px 8px rgba(0, 0, 0, 0.15));
}

[data-theme="dark"] .theme-toggle-btn {
  background: var(--color--background, #3a3a3a);
  border-color: var(--color--foreground--shade-1, #555);
  color: var(--color--text, #ccc);
}

[data-theme="dark"] .theme-toggle-btn:hover {
  background: var(--color--background--light-1, #444);
}

/* Toggle Switch */
.switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .3s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: .3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: #10B981;
}

input:checked + .slider:before {
  transform: translateX(20px);
}

/* Node Properties View */
.property-group {
  margin-bottom: 20px;
}

.property-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  font-size: 14px;
}

.property-input,
.property-select {
  width: 100%;
  padding: 8px 12px;
  background: var(--color--background--light-2, white);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  border-radius: 6px;
  font-size: 14px;
  color: var(--color--text--shade-1, #333);
}

.property-input:disabled {
  background: var(--color--background, #f5f5f5);
  cursor: not-allowed;
}

[data-theme="dark"] .property-input,
[data-theme="dark"] .property-select {
  background: var(--color--background, #3a3a3a);
  border-color: var(--color--foreground--shade-1, #555);
  color: var(--color--text, #ccc);
}

[data-theme="dark"] .property-input:disabled {
  background: var(--color--background--light-1, #2d2d2d);
}

/* Empty State */
.empty-view {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.empty-state {
  text-align: center;
  color: var(--color--text, #666);
}

.empty-state p {
  margin: 8px 0;
  font-size: 14px;
}

.empty-state .hint {
  color: var(--color--text--tint-1, #999);
  font-size: 12px;
}

[data-theme="dark"] .empty-state {
  color: var(--color--text, #aaa);
}

[data-theme="dark"] .empty-state .hint {
  color: var(--color--text--tint-1, #777);
}
</style>
