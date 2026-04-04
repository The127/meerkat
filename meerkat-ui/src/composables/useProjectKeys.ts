import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { PaginatedResponse, ProjectKey } from '@/lib/types'

export function useProjectKeys(slug: Ref<string | undefined>, options?: {
  status?: Ref<string | undefined>
  search?: Ref<string | undefined>
  limit?: Ref<number>
  offset?: Ref<number>
}) {
  const status = computed(() => options?.status ? options.status.value : 'active')
  const search = computed(() => options?.search?.value)
  const limit = computed(() => options?.limit?.value ?? 20)
  const offset = computed(() => options?.offset?.value ?? 0)

  return useQuery({
    queryKey: ['projectKeys', slug, status, search, limit, offset],
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('limit', String(limit.value))
      params.set('offset', String(offset.value))
      if (status.value) params.set('status', status.value)
      if (search.value) params.set('search', search.value)
      return api<PaginatedResponse<ProjectKey>>(`/api/v1/projects/${slug.value}/keys?${params}`)
    },
    enabled: computed(() => !!slug.value),
  })
}
