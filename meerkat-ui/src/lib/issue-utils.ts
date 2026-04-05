import type { Issue } from '@/lib/types'

export { formatRelativeTime } from '@/lib/date-utils'

export function levelVariant(level: Issue['level']) {
  switch (level) {
    case 'fatal':
    case 'error':
      return 'error' as const
    case 'warning':
      return 'warning' as const
    case 'info':
      return 'default' as const
    case 'debug':
      return 'secondary' as const
  }
}

export function statusVariant(status: Issue['status']) {
  switch (status) {
    case 'unresolved':
      return 'destructive' as const
    case 'resolved':
      return 'success' as const
    case 'ignored':
      return 'secondary' as const
    case 'regressed':
      return 'warning' as const
  }
}
