import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { ProjectRole } from '@/lib/types'

export function useCreateProjectRole() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ slug, name, permissions }: { slug: string; name: string; permissions: string[] }) =>
      api<ProjectRole>(`/api/v1/projects/${slug}/roles`, {
        method: 'POST',
        body: JSON.stringify({ name, permissions }),
      }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['projectRoles', variables.slug] })
    },
  })
}
