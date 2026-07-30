import { ref, watch } from 'vue'

export function useFileBrowser() {
  const items = ref([])
  const loading = ref(false)
  const error = ref(null)
  const currentPath = ref([])
  const searchKeyword = ref('')
  const sortBy = ref('name')
  const allItems = ref([])

  async function loadDir(pathArray = []) {
    loading.value = true
    error.value = null
    try {
      const response = await fetch('/api/dir', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: pathArray })
      })
      
      if (!response.ok) {
        throw new Error('加载目录失败')
      }
      
      let data = await response.json()
      data = Array.isArray(data) ? data : [data]
      data = data.map(item => ({
        ...item,
        path: item.path ? item.path.replace(/\\/g, '/') : item.path
      }))
      
      allItems.value = data
      sortAllItems()
      currentPath.value = pathArray
    } catch (e) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  function filterItems() {
    if (!searchKeyword.value.trim()) {
      items.value = allItems.value
      return
    }
    const keyword = searchKeyword.value.toLowerCase()
    items.value = allItems.value.filter(item => 
      item.name.toLowerCase().includes(keyword)
    )
  }

  function sortAllItems() {
    allItems.value.sort((a, b) => {
      if (a.type === 'directory' && b.type !== 'directory') return -1
      if (a.type !== 'directory' && b.type === 'directory') return 1
      if (sortBy.value === 'name') {
        return a.name.localeCompare(b.name, 'zh-CN')
      } else {
        return new Date(b.mtime) - new Date(a.mtime)
      }
    })
    filterItems()
  }

  function goBack() {
    if (currentPath.value.length > 0) {
      const parentPath = currentPath.value.slice(0, -1)
      loadDir(parentPath)
    }
  }

  return {
    items,
    loading,
    error,
    currentPath,
    searchKeyword,
    sortBy,
    allItems,
    loadDir,
    filterItems,
    sortAllItems,
    goBack
  }
}