import type { Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { ProjectMember } from '@/lib/types'

export function useProjectMembers(slug: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['projectMembers', slug],
    queryFn: () => api<ProjectMember[]>(`/api/v1/projects/${slug.value}/members`),
    enabled: () => !!slug.value,
  })
}
