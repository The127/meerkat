import { UserManager, WebStorageStateStore, type UserManagerSettings } from 'oidc-client-ts'
import type { OidcConfig } from './types'

export function createUserManager(oidcConfig: OidcConfig): UserManager {
  const origin = window.location.origin

  const settings: UserManagerSettings = {
    authority: oidcConfig.issuer_url,
    client_id: oidcConfig.client_id,
    redirect_uri: `${origin}/auth/callback`,
    post_logout_redirect_uri: `${origin}/auth/login`,
    response_type: 'code',
    scope: 'openid profile email',
    automaticSilentRenew: true,
    userStore: new WebStorageStateStore({ store: sessionStorage }),
    metadataUrl: discoveryUrl(oidcConfig),
  }

  return new UserManager(settings)
}

/**
 * Returns the OIDC discovery URL.
 * Uses `discovery_url` from the backend if provided,
 * otherwise derives `{issuer}/.well-known/openid-configuration`.
 */
function discoveryUrl(config: OidcConfig): string {
  if (config.discovery_url) {
    return config.discovery_url
  }
  const issuer = config.issuer_url.replace(/\/$/, '')
  return `${issuer}/.well-known/openid-configuration`
}
