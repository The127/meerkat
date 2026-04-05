import { computed, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { PaginatedResponse, Member } from '@/lib/types'

export function useMembers(options?: {
  search?: Ref<string | undefined>
  role?: Ref<string | undefined>
  limit?: Ref<number>
  offset?: Ref<number>
}) {
  const search = computed(() => options?.search?.value)
  const role = computed(() => options?.role?.value)
  const limit = computed(() => options?.limit?.value ?? 20)
  const offset = computed(() => options?.offset?.value ?? 0)

  return useQuery({
    queryKey: ['members', search, role, limit, offset],
    queryFn: () => {
      const params = new URLSearchParams()
      params.set('limit', String(limit.value))
      params.set('offset', String(offset.value))
      if (search.value) params.set('search', search.value)
      if (role.value) params.set('role', role.value)
      return api<PaginatedResponse<Member>>(`/api/v1/members?${params}`)
    },
  })
}
