import { useMutation } from '@tanstack/vue-query'
import { useRouter } from 'vue-router'
import { api } from '@/lib/api'

export function useDeleteOrganization() {
  const router = useRouter()

  return useMutation({
    mutationFn: () =>
      api<void>('/api/v1/organization', {
        method: 'DELETE',
      }),
    onSuccess: () => {
      router.push({ name: 'org-deleted' })
    },
  })
}
