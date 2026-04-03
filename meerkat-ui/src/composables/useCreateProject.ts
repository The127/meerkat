import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { api } from '@/lib/api'

interface CreateProjectRequest {
  organization_id: string
  name: string
  slug: string
}

interface CreateProjectResponse {
  id: string
}

export function useCreateProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateProjectRequest) =>
      api<CreateProjectResponse>('/api/v1/projects', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      queryClient.invalidateQueries({ queryKey: ['currentUser'] })
    },
  })
}
