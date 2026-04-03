import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { OidcConfigListItem } from '@/lib/types'

export function useOidcConfigs() {
  return useQuery({
    queryKey: ['oidcConfigs'],
    queryFn: () => api<OidcConfigListItem[]>('/api/v1/organization/oidc-configs'),
  })
}
