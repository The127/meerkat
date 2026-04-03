import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useDeleteOidcConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (configId: string) =>
      api<void>(`/api/v1/organization/oidc-configs/${configId}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidcConfigs'] })
    },
  })
}
