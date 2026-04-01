import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { PaginatedResponse, Project } from '@/lib/types'

export function useProjects(options?: {
  limit?: Ref<number>
  offset?: Ref<number>
  search?: Ref<string | undefined>
}) {
  const limit = computed(() => options?.limit?.value ?? 20)
  const offset = computed(() => options?.offset?.value ?? 0)
  const search = computed(() => options?.search?.value)

  return useQuery({
    queryKey: ['projects', limit, offset, search],
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('limit', String(limit.value))
      params.set('offset', String(offset.value))
      if (search.value) params.set('search', search.value)
      return api<PaginatedResponse<Project>>(`/api/v1/projects?${params}`)
    },
  })
}
