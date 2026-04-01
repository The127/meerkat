import { type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { Project } from '@/lib/types'

export function useProject(slug: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['project', slug],
    queryFn: () => api<Project>(`/api/v1/projects/${slug.value}`),
    enabled: () => !!slug.value,
  })
}
