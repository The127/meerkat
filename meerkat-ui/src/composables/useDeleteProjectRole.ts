import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useDeleteProjectRole() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, roleId }: { slug: string; roleId: string }) =>
      api<void>(`/api/v1/projects/${slug}/roles/${roleId}`, { method: 'DELETE' }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['projectRoles', variables.slug] })
    },
  })
}
