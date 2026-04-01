import { UserManager, WebStorageStateStore, type UserManagerSettings } from 'oidc-client-ts'
import type { OidcConfig } from './types'

export function createUserManager(oidcConfig: OidcConfig): UserManager {
  const origin = window.location.origin

  const settings: UserManagerSettings = {
    authority: oidcConfig.issuer_url,
    client_id: oidcConfig.client_id,
    redirect_uri: `${origin}/callback`,
    post_logout_redirect_uri: `${origin}/login`,
    response_type: 'code',
    scope: 'openid profile email',
    automaticSilentRenew: true,
    // Keep tokens in session storage so they survive page refresh but not new tabs
    userStore: new WebStorageStateStore({ store: sessionStorage }),
  }

  return new UserManager(settings)
}
