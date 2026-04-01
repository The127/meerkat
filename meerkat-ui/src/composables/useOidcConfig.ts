import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { OidcConfig } from '@/lib/types'

export function useOidcConfig() {
  return useQuery({
    queryKey: ['oidc-config'],
    queryFn: () => api<OidcConfig>('/api/v1/oidc'),
    staleTime: Infinity,
    retry: 2,
  })
}
