import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

// Lazy load components for better performance
const WorkflowEditor = () => import('@/views/WorkflowEditor.vue')
const WorkflowList = () => import('@/views/WorkflowList.vue')
const WorkflowsHome = () => import('@/views/WorkflowsHome.vue')

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: WorkflowsHome,
    meta: {
      title: 'Workflows - Agent Builder'
    }
  },
  {
    path: '/workflows',
    name: 'WorkflowList',
    component: WorkflowList,
    meta: {
      title: 'Workflows'
    }
  },
  {
    path: '/workflow/new',
    name: 'NewWorkflow',
    component: WorkflowEditor,
    meta: {
      title: 'New Workflow'
    }
  },
  {
    path: '/workflow/:id',
    name: 'EditWorkflow',
    component: WorkflowEditor,
    props: true,
    meta: {
      title: 'Edit Workflow'
    }
  },
  {
    path: '/workflow/:id/view',
    name: 'ViewWorkflow',
    component: WorkflowEditor,
    props: route => ({ 
      id: route.params.id, 
      readOnly: true 
    }),
    meta: {
      title: 'View Workflow'
    }
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    redirect: '/workflows'
  }
]

const router = createRouter({
  history: createWebHistory('/'),
  routes
})

// Navigation guards
router.beforeEach((to, _from, next) => {
  // Set document title
  if (to.meta?.title) {
    document.title = `${to.meta.title} - Agent Builder`
  } else {
    document.title = 'Agent Builder'
  }
  
  next()
})

export default router