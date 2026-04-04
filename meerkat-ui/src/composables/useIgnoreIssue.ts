import { useIssueAction } from './useIssueAction'

export function useIgnoreIssue() {
  return useIssueAction('ignore')
}
