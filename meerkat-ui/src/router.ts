import { createRouter, createWebHistory } from 'vue-router'
import { useAuth } from '@/composables/useAuth'

const PUBLIC_ROUTES = new Set(['login', 'callback', 'logout', 'org-deleted'])

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: () => import('@/layouts/AppLayout.vue'),
      children: [
        {
          path: '',
          name: 'dashboard',
          component: () => import('@/pages/Home.vue'),
        },
        {
          path: 'projects',
          name: 'projects',
          component: () => import('@/pages/projects/ProjectList.vue'),
          children: [
            {
              path: 'new',
              name: 'projects-new',
              component: () => import('@/pages/projects/CreateProjectDialog.vue'),
            },
          ],
        },
        {
          path: 'projects/:slug',
          name: 'project-dashboard',
          component: () => import('@/pages/projects/ProjectDetail.vue'),
        },
        {
          path: 'profile',
          name: 'profile',
          component: () => import('@/pages/Profile.vue'),
        },
        {
          path: 'settings',
          name: 'org-settings',
          component: () => import('@/pages/OrgSettings.vue'),
        },
      ],
    },
    {
      path: '/org-deleted',
      name: 'org-deleted',
      component: () => import('@/pages/OrgDeleted.vue'),
    },
    {
      path: '/auth/login',
      name: 'login',
      component: () => import('@/pages/Login.vue'),
    },
    {
      path: '/auth/callback',
      name: 'callback',
      component: () => import('@/pages/Callback.vue'),
    },
    {
      path: '/auth/logout',
      name: 'logout',
      component: () => import('@/pages/Logout.vue'),
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: () => import('@/pages/NotFound.vue'),
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

  if (PUBLIC_ROUTES.has(routeName) || routeName === 'not-found') {
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
