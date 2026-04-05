import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { MemberAccess } from '@/lib/types'

export function useMemberAccess(memberId: Ref<string | undefined>) {
  return useQuery({
    queryKey: ['memberAccess', memberId],
    queryFn: () => api<MemberAccess>(`/api/v1/members/${memberId.value}/access`),
    enabled: computed(() => !!memberId.value),
  })
}
