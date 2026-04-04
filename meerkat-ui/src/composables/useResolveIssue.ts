import { useIssueAction } from './useIssueAction'

export function useResolveIssue() {
  return useIssueAction('resolve')
}
