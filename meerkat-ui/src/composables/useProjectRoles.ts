import type { Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { ProjectRole } from '@/lib/types'

export function useProjectRoles(slug: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['projectRoles', slug],
    queryFn: () => api<ProjectRole[]>(`/api/v1/projects/${slug.value}/roles`),
    enabled: () => !!slug.value,
  })
}
