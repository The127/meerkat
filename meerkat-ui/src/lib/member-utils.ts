export function roleVariant(role: string) {
  switch (role) {
    case 'owner': return 'destructive' as const
    case 'admin': return 'warning' as const
    default: return 'secondary' as const
  }
}

export function formatPermission(perm: string): string {
  return perm.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}
