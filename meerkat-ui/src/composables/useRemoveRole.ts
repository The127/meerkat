import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useRemoveRole() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      slug,
      memberId,
      roleId,
    }: {
      slug: string
      memberId: string
      roleId: string
    }) =>
      api<void>(`/api/v1/projects/${slug}/members/${memberId}/roles/${roleId}`, {
        method: 'DELETE',
      }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['projectMembers', variables.slug] })
    },
  })
}
