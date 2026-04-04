import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useUpdateProjectKeyRateLimit() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, keyId, rateLimit }: { slug: string; keyId: string; rateLimit: number | null }) =>
      api<void>(`/api/v1/projects/${slug}/keys/${keyId}/rate-limit`, {
        method: 'POST',
        body: JSON.stringify({ rate_limit: rateLimit }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projectKeys'] })
    },
  })
}
