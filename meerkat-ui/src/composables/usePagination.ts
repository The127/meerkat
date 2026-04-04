import { ref, computed } from 'vue'

export function usePagination(pageSize = 20) {
  const offset = ref(0)
  const limit = computed(() => pageSize)

  function prevPage() {
    offset.value = Math.max(0, offset.value - pageSize)
  }

  function nextPage() {
    offset.value += pageSize
  }

  function reset() {
    offset.value = 0
  }

  function pageInfo(total: number) {
    return {
      currentPage: Math.floor(offset.value / pageSize) + 1,
      totalPages: Math.ceil(total / pageSize),
      hasPrev: offset.value > 0,
      hasNext: offset.value + pageSize < total,
    }
  }

  return { offset, limit, prevPage, nextPage, reset, pageInfo }
}
