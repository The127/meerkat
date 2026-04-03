import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useActivateOidcConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (configId: string) =>
      api<void>(`/api/v1/organization/oidc-configs/${configId}/activate`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidcConfigs'] })
    },
  })
}
