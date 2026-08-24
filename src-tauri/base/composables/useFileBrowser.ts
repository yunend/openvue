import { ref, watch, type Ref } from 'vue'

export function useFileBrowser() {
  const items = ref<FileItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const currentPath = ref<string[]>([])
  const searchKeyword = ref('')
  const sortBy = ref<'name' | 'time'>('name')
  const allItems = ref<FileItem[]>([])

  async function loadDir(pathArray: string[] = []): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await fetch('/api/dir', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: pathArray }),
      })

      if (!response.ok) {
        throw new Error('加载目录失败')
      }

      let data: FileItem[] = await response.json()
      data = Array.isArray(data) ? data : [data]
      data = data.map((item) => ({
        ...item,
        path: item.path ? item.path.replace(/\\/g, '/') : item.path,
      }))

      allItems.value = data
      sortAllItems()
      currentPath.value = pathArray
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  function filterItems(): void {
    if (!searchKeyword.value.trim()) {
      items.value = allItems.value
      return
    }
    const keyword = searchKeyword.value.toLowerCase()
    items.value = allItems.value.filter((item) =>
      item.name.toLowerCase().includes(keyword)
    )
  }

  function sortAllItems(): void {
    allItems.value.sort((a, b) => {
      if (a.type === 'directory' && b.type !== 'directory') return -1
      if (a.type !== 'directory' && b.type === 'directory') return 1
      if (sortBy.value === 'name') {
        return a.name.localeCompare(b.name, 'zh-CN')
      } else {
        return new Date(b.mtime).getTime() - new Date(a.mtime).getTime()
      }
    })
    filterItems()
  }

  function goBack(): void {
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
    goBack,
  }
}