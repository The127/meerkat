import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

export function useRenameOrganization() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (name: string) =>
      api<void>('/api/v1/organization/rename', {
        method: 'POST',
        body: JSON.stringify({ name }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['organization'] })
    },
  })
}
