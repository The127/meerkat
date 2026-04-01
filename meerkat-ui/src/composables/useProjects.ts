import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { PaginatedResponse, Project } from '@/lib/types'

export function useProjects(options?: { limit?: Ref<number>; offset?: Ref<number> }) {
  const limit = computed(() => options?.limit?.value ?? 20)
  const offset = computed(() => options?.offset?.value ?? 0)

  return useQuery({
    queryKey: ['projects', limit, offset],
    queryFn: () =>
      api<PaginatedResponse<Project>>(
        `/api/v1/projects?limit=${limit.value}&offset=${offset.value}`,
      ),
  })
}
