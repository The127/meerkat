import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { api } from '@/lib/api'
import type { CurrentUser } from '@/lib/types'

export function useCurrentUser() {
  const query = useQuery({
    queryKey: ['currentUser'],
    queryFn: () => api<CurrentUser>('/api/v1/me'),
    staleTime: 5 * 60 * 1000,
  })

  function hasOrgPermission(permission: string): boolean {
    return query.data.value?.org_permissions.includes(permission) ?? false
  }

  function hasProjectPermission(slug: string, permission: string): boolean {
    return query.data.value?.project_permissions[slug]?.includes(permission) ?? false
  }

  return {
    ...query,
    hasOrgPermission,
    hasProjectPermission,
    canRenameOrg: computed(() => hasOrgPermission('org_rename')),
    canDeleteOrg: computed(() => hasOrgPermission('org_delete')),
    canManageOidc: computed(() => hasOrgPermission('org_manage_oidc')),
    canManageMembers: computed(() => hasOrgPermission('org_manage_members')),
    canCreateProject: computed(() => hasOrgPermission('org_create_project')),
  }
}
