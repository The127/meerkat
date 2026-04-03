import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

interface AddOidcConfigRequest {
  name: string
  client_id: string
  issuer_url: string
  audience: string
  discovery_url?: string
  sub_claim: string
  name_claim: string
  role_claim: string
  owner_values: string[]
  admin_values: string[]
  member_values: string[]
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
