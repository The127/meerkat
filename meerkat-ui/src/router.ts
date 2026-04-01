import { createRouter, createWebHistory } from 'vue-router'
import { useAuth } from '@/composables/useAuth'

const PUBLIC_ROUTES = new Set(['login', 'callback', 'logout'])

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/pages/Home.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/pages/Login.vue'),
    },
    {
      path: '/callback',
      name: 'callback',
      component: () => import('@/pages/Callback.vue'),
    },
    {
      path: '/logout',
      name: 'logout',
      component: () => import('@/pages/Logout.vue'),
    },
    ...(import.meta.env.DEV
      ? [
          {
            path: '/kitchen-sink',
            name: 'kitchen-sink',
            component: () => import('@/pages/KitchenSink.vue'),
          },
        ]
      : []),
  ],
})

router.beforeEach((to) => {
  const routeName = typeof to.name === 'string' ? to.name : ''

  if (PUBLIC_ROUTES.has(routeName)) {
    return true
  }

  const { isAuthenticated, isLoading } = useAuth()

  if (isLoading.value) {
    return true
  }

  if (!isAuthenticated.value) {
    return { name: 'login' }
  }

  return true
})

export default router
