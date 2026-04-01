import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useProject } from './useProject'

export function useCurrentProject() {
  const route = useRoute()

  const slug = computed(() => {
    const param = route.params.slug
    return typeof param === 'string' ? param : undefined
  })

  const { data: currentProject, isLoading } = useProject(slug)

  return { slug, currentProject, isLoading }
}
