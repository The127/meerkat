import { useIssueAction } from './useIssueAction'

export function useReopenIssue() {
  return useIssueAction('reopen')
}
