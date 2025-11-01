<script setup lang="ts">
import { onMounted } from 'vue'
import { useUIStore } from '@/stores'

const uiStore = useUIStore()

onMounted(() => {
  // Initialize theme
  const savedTheme = localStorage.getItem('agent-builder-theme') as 'light' | 'dark' || 'light'
  uiStore.setTheme(savedTheme)
  
  // Watch for theme changes and save to localStorage
  uiStore.$subscribe((_mutation, state) => {
    localStorage.setItem('agent-builder-theme', state.theme)
  })
})
</script>

<template>
  <div id="app" :data-theme="uiStore.theme" class="h-screen w-screen">
    <!-- Global Loading Overlay -->
    <div v-if="uiStore.loading" class="loading-overlay">
      <div class="loading-spinner">Loading...</div>
    </div>

    <!-- Main Router View -->
    <router-view />

    <!-- Global Notifications (will be enhanced with Element Plus later) -->
    <div class="notifications-container">
      <div
        v-for="notification in uiStore.notifications"
        :key="notification.id"
        :class="['notification', `notification-${notification.type}`]"
        @click="uiStore.removeNotification(notification.id)"
      >
        {{ notification.message }}
      </div>
    </div>
  </div>
</template>

<style>
/* Global Styles */
* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

#app {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.loading-spinner {
  padding: 20px;
  background: rgba(0, 0, 0, 0.8);
  border-radius: 8px;
}

.notifications-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9998;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.notification {
  padding: 12px 16px;
  border-radius: 6px;
  color: white;
  cursor: pointer;
  min-width: 200px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.notification-success { background-color: #67C23A; }
.notification-error { background-color: #F56C6C; }
.notification-warning { background-color: #E6A23C; }
.notification-info { background-color: #909399; }

/* Theme Variables */
:root {
  --color-primary: #409EFF;
  --color-success: #67C23A;
  --color-warning: #E6A23C;
  --color-danger: #F56C6C;
  --color-info: #909399;
  
  --color-background: #ffffff;
  --color-background-light: #f8f9fa;
  --color-background-hover: #f1f3f4;
  
  --color-text-primary: #2c3e50;
  --color-text-secondary: #5a6c7d;
  --color-text-tertiary: #8b9bb3;
  
  --color-border: #e1e5e9;
  --color-border-light: #f0f2f5;
  
  --shadow-light: 0 2px 8px rgba(0, 0, 0, 0.1);
  --shadow-medium: 0 4px 16px rgba(0, 0, 0, 0.15);
}

[data-theme="dark"] {
  --color-background: #1a1a1a;
  --color-background-light: #2d2d2d;
  --color-background-hover: #404040;
  
  --color-text-primary: #ffffff;
  --color-text-secondary: #b0b7c3;
  --color-text-tertiary: #8b9bb3;
  
  --color-border: #404040;
  --color-border-light: #505050;
  
  --shadow-light: 0 2px 8px rgba(0, 0, 0, 0.3);
  --shadow-medium: 0 4px 16px rgba(0, 0, 0, 0.4);
}

/* Scrollbar Styling */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--color-background-light);
}

::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}
</style>
