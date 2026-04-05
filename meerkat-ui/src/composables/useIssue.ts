import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { Issue } from '@/lib/types'

export function useIssue(slug: Ref<string | undefined>, issueNumber: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['issue', slug, issueNumber],
    queryFn: () => api<Issue>(`/api/v1/projects/${slug.value}/issues/${issueNumber.value}`),
    enabled: computed(() => !!slug.value && !!issueNumber.value),
  })
}
