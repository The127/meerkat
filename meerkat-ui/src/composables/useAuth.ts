import { ref, computed, type Ref } from 'vue'
import { type User, type UserManager } from 'oidc-client-ts'

const user: Ref<User | null> = ref(null)
const isLoading = ref(true)
let userManager: UserManager | null = null

export function initAuth(manager: UserManager) {
  userManager = manager

  manager.events.addUserLoaded((u) => {
    user.value = u
  })

  manager.events.addUserUnloaded(() => {
    user.value = null
  })

  manager.events.addSilentRenewError(() => {
    user.value = null
  })

  // Try to load existing session
  manager.getUser().then((u) => {
    user.value = u && !u.expired ? u : null
    isLoading.value = false
  })
}

export function useAuth() {
  const isAuthenticated = computed(() => !!user.value && !user.value.expired)

  async function login() {
    if (!userManager) throw new Error('Auth not initialized')
    await userManager.signinRedirect()
  }

  async function handleCallback(): Promise<User> {
    if (!userManager) throw new Error('Auth not initialized')
    const u = await userManager.signinRedirectCallback()
    user.value = u
    return u
  }

  async function logout() {
    if (!userManager) throw new Error('Auth not initialized')
    await userManager.signoutRedirect()
  }

  async function getToken(): Promise<string | null> {
    if (!user.value || user.value.expired) return null
    return user.value.access_token
  }

  return {
    user: computed(() => user.value),
    isAuthenticated,
    isLoading: computed(() => isLoading.value),
    login,
    logout,
    handleCallback,
    getToken,
  }
}
