import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useResolveIssue() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, issueId }: { slug: string; issueId: string }) =>
      api<void>(`/api/v1/projects/${slug}/issues/${issueId}/resolve`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['issues'] })
    },
  })
}
