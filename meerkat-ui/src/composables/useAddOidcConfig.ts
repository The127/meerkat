import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { ClaimMapping } from '@/lib/types'

interface AddOidcConfigRequest {
  name: string
  client_id: string
  issuer_url: string
  audience: string
  discovery_url?: string
  claim_mapping: ClaimMapping
}

export function useAddOidcConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: AddOidcConfigRequest) =>
      api<{ id: string }>('/api/v1/organization/oidc-configs', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidcConfigs'] })
    },
  })
}
