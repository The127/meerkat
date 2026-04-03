import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useRenameProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, name }: { slug: string; name: string }) =>
      api<void>(`/api/v1/projects/${slug}/rename`, {
        method: 'POST',
        body: JSON.stringify({ name }),
      }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['project', variables.slug] })
      queryClient.invalidateQueries({ queryKey: ['projects'] })
    },
  })
}
