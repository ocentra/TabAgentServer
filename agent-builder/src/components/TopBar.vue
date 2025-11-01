<template>
  <div class="top-bar">
    <div class="top-bar-left">
      <!-- Agent Builder Branding -->
      <div class="branding">
        🤖 Agent Builder
      </div>
      
      <!-- Workflow Title (Double-click to edit) -->
      <div v-if="!editingTitle" class="workflow-title-display" @dblclick="startEditTitle">
        {{ workflowTitle || 'Untitled Workflow' }}
      </div>
      <input 
        v-else
        ref="titleInput"
        v-model="workflowTitle" 
        class="workflow-title-input"
        placeholder="Untitled Workflow"
        @blur="finishEditTitle"
        @keyup.enter="finishEditTitle"
      />
    </div>
    
    <div class="top-bar-center">
      <!-- Navigation Tabs -->
      <div class="nav-tabs">
        <button 
          :class="['tab', { active: activeTab === 'editor' }]"
          @click="activeTab = 'editor'"
        >
          Editor
        </button>
        <button 
          :class="['tab', { active: activeTab === 'executions' }]"
          @click="activeTab = 'executions'"
        >
          Executions
        </button>
        <button 
          :class="['tab', { active: activeTab === 'evaluations' }]"
          @click="activeTab = 'evaluations'"
        >
          Evaluations
        </button>
      </div>
    </div>
    
    <div class="top-bar-right">
      <!-- Active/Inactive Toggle -->
      <div class="status-toggle">
        <span class="status-label">{{ isActive ? 'Active' : 'Inactive' }}</span>
        <label class="switch">
          <input type="checkbox" v-model="isActive" @change="toggleStatus">
          <span class="slider"></span>
        </label>
      </div>
      
      <!-- Share Button -->
      <button class="btn btn-secondary" @click="handleShare">
        Share
      </button>
      
      <!-- Save Button -->
      <button class="btn btn-primary" @click="handleSave">
        Save
      </button>
      
      <!-- More Options -->
      <button class="btn btn-icon" @click="showMoreMenu = !showMoreMenu">
        ⋯
      </button>
      
      <!-- More Menu Dropdown -->
      <div v-if="showMoreMenu" class="more-menu">
        <button @click="handleDuplicate">Duplicate</button>
        <button @click="handleDelete">Delete</button>
        <button @click="handleExport">Export</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'

const workflowTitle = ref('Untitled Workflow')
const activeTab = ref('editor')
const isActive = ref(false)
const showMoreMenu = ref(false)
const editingTitle = ref(false)
const titleInput = ref<HTMLInputElement | null>(null)

const emit = defineEmits(['save', 'share', 'duplicate', 'delete', 'export', 'settings'])

const startEditTitle = async () => {
  editingTitle.value = true
  await nextTick()
  titleInput.value?.focus()
  titleInput.value?.select()
}

const finishEditTitle = () => {
  editingTitle.value = false
  console.log('Saving title:', workflowTitle.value)
  // TODO: Emit save event
}

const toggleStatus = () => {
  console.log('Workflow status:', isActive.value ? 'Active' : 'Inactive')
  // TODO: Emit status change
}

const handleSave = () => {
  emit('save')
  console.log('Save workflow')
}

const handleShare = () => {
  emit('share')
  console.log('Share workflow')
}

const handleDuplicate = () => {
  showMoreMenu.value = false
  emit('duplicate')
  console.log('Duplicate workflow')
}

const handleDelete = () => {
  showMoreMenu.value = false
  emit('delete')
  console.log('Delete workflow')
}

const handleExport = () => {
  showMoreMenu.value = false
  emit('export')
  console.log('Export workflow')
}
</script>

<style scoped>
.top-bar {
  height: 60px;
  background: var(--color--background--light-2, white);
  border-bottom: 1px solid var(--color--foreground--shade-2, #ddd);
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 20px;
  z-index: 10;
}

[data-theme="dark"] .top-bar {
  background: var(--color--background--light-1, #2d2d2d);
  border-bottom-color: var(--color--foreground--shade-1, #444);
}

.top-bar-left {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 16px;
}

.branding {
  font-size: 18px;
  font-weight: 700;
  color: var(--color--text--shade-1, #333);
  display: flex;
  align-items: center;
  gap: 8px;
}

[data-theme="dark"] .branding {
  color: var(--color--text--shade-1, #e0e0e0);
}

.workflow-title-display {
  font-size: 16px;
  font-weight: 500;
  color: var(--color--text--shade-1, #555);
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
  user-select: none;
}

.workflow-title-display:hover {
  background: var(--color--background, #f5f5f5);
}

[data-theme="dark"] .workflow-title-display {
  color: var(--color--text, #ccc);
}

[data-theme="dark"] .workflow-title-display:hover {
  background: var(--color--background, #3a3a3a);
}

.workflow-title-input {
  font-size: 16px;
  font-weight: 500;
  border: 2px solid #ff6d5a;
  background: var(--color--background--light-2, white);
  color: var(--color--text--shade-1, #333);
  padding: 8px 12px;
  border-radius: 4px;
  transition: background 0.2s;
  width: 300px;
  max-width: 100%;
}

.workflow-title-input:focus {
  outline: none;
  box-shadow: 0 0 0 3px rgba(255, 109, 90, 0.1);
}

[data-theme="dark"] .workflow-title-input {
  background: var(--color--background, #3a3a3a);
  color: var(--color--text--shade-1, #e0e0e0);
  border-color: #ff6d5a;
}

.top-bar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.nav-tabs {
  display: flex;
  gap: 4px;
  background: var(--color--background, #f5f5f5);
  border-radius: 8px;
  padding: 4px;
}

[data-theme="dark"] .nav-tabs {
  background: var(--color--background, #3a3a3a);
}

.tab {
  padding: 8px 20px;
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: var(--color--text, #666);
  transition: all 0.2s;
}

.tab:hover {
  color: var(--color--text--shade-1, #333);
}

.tab.active {
  background: var(--color--background--light-2, white);
  color: var(--color--text--shade-1, #333);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

[data-theme="dark"] .tab {
  color: var(--color--text, #aaa);
}

[data-theme="dark"] .tab:hover {
  color: var(--color--text--shade-1, #e0e0e0);
}

[data-theme="dark"] .tab.active {
  background: var(--color--background--light-1, #2d2d2d);
  color: var(--color--text--shade-1, #e0e0e0);
}

.top-bar-right {
  flex: 1;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  position: relative;
}

/* Status Toggle */
.status-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-label {
  font-size: 14px;
  color: var(--color--text, #666);
}

[data-theme="dark"] .status-label {
  color: var(--color--text, #aaa);
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

/* Buttons */
.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-secondary {
  background: transparent;
  color: var(--color--text--shade-1, #333);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
}

.btn-secondary:hover {
  background: var(--color--background, #f5f5f5);
}

[data-theme="dark"] .btn-secondary {
  color: var(--color--text--shade-1, #e0e0e0);
  border-color: var(--color--foreground--shade-1, #555);
}

[data-theme="dark"] .btn-secondary:hover {
  background: var(--color--background, #3a3a3a);
}

.btn-primary {
  background: #ff6d5a;
  color: white;
}

.btn-primary:hover {
  background: #ff5a45;
}

.btn-icon {
  width: 36px;
  height: 36px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--color--text--shade-1, #333);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  font-size: 20px;
}

.btn-icon:hover {
  background: var(--color--background, #f5f5f5);
}

[data-theme="dark"] .btn-icon {
  color: var(--color--text--shade-1, #e0e0e0);
  border-color: var(--color--foreground--shade-1, #555);
}

[data-theme="dark"] .btn-icon:hover {
  background: var(--color--background, #3a3a3a);
}

/* More Menu */
.more-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 8px;
  background: var(--color--background--light-2, white);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  border-radius: 8px;
  box-shadow: var(--shadow--dark, 0 4px 12px rgba(0, 0, 0, 0.15));
  min-width: 180px;
  z-index: 100;
}

[data-theme="dark"] .more-menu {
  background: var(--color--background--light-1, #2d2d2d);
  border-color: var(--color--foreground--shade-1, #444);
}

.more-menu button {
  width: 100%;
  padding: 12px 16px;
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  color: var(--color--text--shade-1, #333);
  font-size: 14px;
  transition: background 0.2s;
}

.more-menu button:hover {
  background: var(--color--background, #f5f5f5);
}

.more-menu button:first-child {
  border-radius: 8px 8px 0 0;
}

.more-menu button:last-child {
  border-radius: 0 0 8px 8px;
}

[data-theme="dark"] .more-menu button {
  color: var(--color--text--shade-1, #e0e0e0);
}

[data-theme="dark"] .more-menu button:hover {
  background: var(--color--background, #3a3a3a);
}
</style>

