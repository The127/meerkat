import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useUpdateProjectRole() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      slug,
      roleId,
      name,
      permissions,
    }: {
      slug: string
      roleId: string
      name: string
      permissions: string[]
    }) =>
      api<void>(`/api/v1/projects/${slug}/roles/${roleId}`, {
        method: 'PUT',
        body: JSON.stringify({ name, permissions }),
      }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['projectRoles', variables.slug] })
    },
  })
}
