import type { App } from 'vue'
import type { UserManager } from 'oidc-client-ts'
import { initAuth, useAuth } from '@/composables/useAuth'
import { setTokenProvider } from '@/lib/api'

export const authPlugin = {
  install(_app: App, userManager: UserManager) {
    initAuth(userManager)

    const { getToken } = useAuth()
    setTokenProvider(getToken)
  },
}
