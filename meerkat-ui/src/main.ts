import { createApp } from 'vue'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import { api } from './lib/api'
import { createUserManager } from './lib/auth'
import { authPlugin } from './plugins/auth'
import type { OidcConfig } from './lib/types'
import './style.css'

async function bootstrap() {
  const app = createApp(App)

  app.use(router)
  app.use(VueQueryPlugin)

  // Fetch OIDC config and initialize auth.
  // If this fails (e.g. no org, backend down), the app still mounts —
  // the login page will show an error via its own useOidcConfig() query.
  try {
    const oidcConfig = await api<OidcConfig>('/api/v1/oidc')
    const userManager = createUserManager(oidcConfig)
    app.use(authPlugin, userManager)
  } catch {
    // Auth will remain uninitialized — login page handles this gracefully
  }

  app.mount('#app')
}

bootstrap()
