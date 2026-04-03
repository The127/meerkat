import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { ClaimMapping } from '@/lib/types'

export function useUpdateOidcClaimMapping() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ configId, claimMapping }: { configId: string; claimMapping: ClaimMapping }) =>
      api<void>(`/api/v1/organization/oidc-configs/${configId}/claim-mapping`, {
        method: 'PUT',
        body: JSON.stringify(claimMapping),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidcConfigs'] })
    },
  })
}
