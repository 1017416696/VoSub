import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Welcome',
      component: () => import('@/views/WelcomePage.vue'),
    },
    {
      path: '/editor',
      name: 'Editor',
      component: () => import('@/views/EditorPage.vue'),
    },
  ],
})

export default router
