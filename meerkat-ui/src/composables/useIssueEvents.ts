import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { PaginatedResponse, Event } from '@/lib/types'

export function useIssueEvents(
  slug: Ref<string | undefined>,
  issueNumber: Ref<string | undefined>,
  options?: {
    limit?: Ref<number>
    offset?: Ref<number>
  },
) {
  const limit = computed(() => options?.limit?.value ?? 20)
  const offset = computed(() => options?.offset?.value ?? 0)

  return useQuery({
    queryKey: ['issue-events', slug, issueNumber, limit, offset],
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('limit', String(limit.value))
      params.set('offset', String(offset.value))
      return api<PaginatedResponse<Event>>(
        `/api/v1/projects/${slug.value}/issues/${issueNumber.value}/events?${params}`,
      )
    },
    enabled: computed(() => !!slug.value && !!issueNumber.value),
  })
}
