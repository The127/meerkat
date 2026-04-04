import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { Issue } from '@/lib/types'

export function useIssue(slug: Ref<string | undefined>, issueId: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['issue', slug, issueId],
    queryFn: () => api<Issue>(`/api/v1/projects/${slug.value}/issues/${issueId.value}`),
    enabled: computed(() => !!slug.value && !!issueId.value),
  })
}
