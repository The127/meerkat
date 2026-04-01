import { ref } from 'vue'

const collapsed = ref(false)

export function useSidebar() {
  function toggle() {
    collapsed.value = !collapsed.value
  }

  return {
    collapsed,
    toggle,
  }
}
