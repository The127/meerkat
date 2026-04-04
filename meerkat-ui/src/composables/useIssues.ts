import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { PaginatedResponse, Issue } from '@/lib/types'

export function useIssues(slug: Ref<string | undefined>, options?: {
  status?: Ref<string | undefined>
  search?: Ref<string | undefined>
}) {
  const status = computed(() => options?.status?.value)
  const search = computed(() => options?.search?.value)

  return useQuery({
    queryKey: ['issues', slug, status, search],
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('limit', '100')
      params.set('offset', '0')
      if (status.value) params.set('status', status.value)
      if (search.value) params.set('search', search.value)
      return api<PaginatedResponse<Issue>>(`/api/v1/projects/${slug.value}/issues?${params}`)
    },
    enabled: computed(() => !!slug.value),
  })
}
