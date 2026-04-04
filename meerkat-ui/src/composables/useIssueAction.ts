import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

type IssueAction = 'resolve' | 'reopen' | 'ignore'

export function useIssueAction(action: IssueAction) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, issueId }: { slug: string; issueId: string }) =>
      api<void>(`/api/v1/projects/${slug}/issues/${issueId}/${action}`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['issues'] })
    },
  })
}
