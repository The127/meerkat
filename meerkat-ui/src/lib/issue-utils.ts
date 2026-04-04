import type { Issue } from '@/lib/types'

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
  }
}

export function formatRelativeTime(iso: string): string {
  const now = Date.now()
  const then = new Date(iso).getTime()
  const diffMs = now - then

  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  return new Date(iso).toLocaleDateString()
}
