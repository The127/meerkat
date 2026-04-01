import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { Organization } from '@/lib/types'

export function useOrganization() {
  return useQuery({
    queryKey: ['organization'],
    queryFn: () => api<Organization>('/api/v1/organization'),
    staleTime: Infinity,
  })
}
