import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useProjects } from './useProjects'

export function useCurrentProject() {
  const route = useRoute()
  const { data: projectsData, isLoading } = useProjects()

  const slug = computed(() => {
    const param = route.params.slug
    return typeof param === 'string' ? param : undefined
  })

  const currentProject = computed(() => {
    if (!slug.value || !projectsData.value) return undefined
    return projectsData.value.items.find((p) => p.slug === slug.value)
  })

  return { slug, currentProject, isLoading }
}
