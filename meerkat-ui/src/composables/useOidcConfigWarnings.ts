import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { OidcConfigWarning } from '@/lib/types'

export function useOidcConfigWarnings(configId: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['oidcConfigWarnings', configId],
    queryFn: () => api<OidcConfigWarning[]>(`/api/v1/organization/oidc-configs/${configId.value}/warnings`),
    enabled: computed(() => !!configId.value),
  })
}
