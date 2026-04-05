import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useDismissOidcConfigWarning() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ configId, warningKey }: { configId: string; warningKey: string }) =>
      api<void>(`/api/v1/organization/oidc-configs/${configId}/warnings/${warningKey}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidcConfigWarnings'] })
    },
  })
}
