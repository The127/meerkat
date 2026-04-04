import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useRevokeProjectKey() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, keyId }: { slug: string; keyId: string }) =>
      api<void>(`/api/v1/projects/${slug}/keys/${keyId}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projectKeys'] })
    },
  })
}
