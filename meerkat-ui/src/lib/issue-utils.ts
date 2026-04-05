import type { Issue } from '@/lib/types'

export { formatRelativeTime } from '@/lib/date-utils'

export function levelVariant(level: Issue['level']): string {
  switch (level) {
    case 'fatal':
    case 'error':
      return 'error'
    case 'warning':
      return 'warning'
    case 'info':
      return 'default'
    case 'debug':
      return 'secondary'
  }
}

export function statusVariant(status: Issue['status']): string {
  switch (status) {
    case 'unresolved':
      return 'destructive'
    case 'resolved':
      return 'success'
    case 'ignored':
      return 'secondary'
    case 'regressed':
      return 'warning'
  }
}
