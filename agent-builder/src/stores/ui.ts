import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useUIStore = defineStore('ui', () => {
  // State
  const sidebarOpen = ref(true)
  const propertiesPanelOpen = ref(true)
  const nodeLibraryOpen = ref(true)
  const executionPanelOpen = ref(false)
  const theme = ref<'light' | 'dark'>('light')
  const loading = ref(false)
  const notifications = ref<Array<{ id: string; type: 'success' | 'error' | 'warning' | 'info'; message: string }>>([])

  // Getters
  const isDarkTheme = computed(() => theme.value === 'dark')
  const hasNotifications = computed(() => notifications.value.length > 0)

  // Actions
  function toggleSidebar() {
    sidebarOpen.value = !sidebarOpen.value
  }

  function togglePropertiesPanel() {
    propertiesPanelOpen.value = !propertiesPanelOpen.value
  }

  function toggleNodeLibrary() {
    nodeLibraryOpen.value = !nodeLibraryOpen.value
  }

  function toggleExecutionPanel() {
    executionPanelOpen.value = !executionPanelOpen.value
  }

  function setTheme(newTheme: 'light' | 'dark') {
    theme.value = newTheme
    // Apply theme to document
    document.documentElement.setAttribute('data-theme', newTheme)
  }

  function setLoading(value: boolean) {
    loading.value = value
  }

  function addNotification(notification: { type: 'success' | 'error' | 'warning' | 'info'; message: string }) {
    const id = Date.now().toString()
    notifications.value.push({ id, ...notification })
    
    // Auto-remove after 5 seconds
    setTimeout(() => {
      removeNotification(id)
    }, 5000)
  }

  function removeNotification(id: string) {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index !== -1) {
      notifications.value.splice(index, 1)
    }
  }

  return {
    // State
    sidebarOpen,
    propertiesPanelOpen,
    nodeLibraryOpen,
    executionPanelOpen,
    theme,
    loading,
    notifications,
    
    // Getters
    isDarkTheme,
    hasNotifications,
    
    // Actions
    toggleSidebar,
    togglePropertiesPanel,
    toggleNodeLibrary,
    toggleExecutionPanel,
    setTheme,
    setLoading,
    addNotification,
    removeNotification
  }
})