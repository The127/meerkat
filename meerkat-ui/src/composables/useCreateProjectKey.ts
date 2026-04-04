import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useCreateProjectKey() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, label }: { slug: string; label: string }) =>
      api<{ id: string }>(`/api/v1/projects/${slug}/keys`, {
        method: 'POST',
        body: JSON.stringify({ label }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projectKeys'] })
    },
  })
}
