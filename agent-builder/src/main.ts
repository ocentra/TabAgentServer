import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

// Tailwind CSS
import './styles/tailwind.css'

// Vue Flow imports
import { VueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
// import { MiniMap } from '@vue-flow/minimap' // TODO: Fix version compatibility

// Vue Flow styles
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'

// Design system + canvas theme
import './styles/index.scss'

import App from './App.vue'
import router from './router'
import { applyVueFlowTheme } from './utils/vue-flow-config'

// Create pinia store
const pinia = createPinia()

// Create app
const app = createApp(App)

// Register Vue Flow components globally
app.component('VueFlow', VueFlow)
app.component('Background', Background)
app.component('Controls', Controls)
// app.component('MiniMap', MiniMap) // TODO: Fix version compatibility

app.use(router)
app.use(pinia)
app.use(ElementPlus)

// Apply Vue Flow theme
applyVueFlowTheme()

app.mount('#app')
