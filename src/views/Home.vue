<template>
  <div class="home">
    <div class="header">
      <h1>oFund</h1>
      <p class="subtitle">基金策略分析工具</p>
    </div>
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="total-count" v-if="funds.length">共 {{ funds.length }} 只基金</span>
        <span class="filtered-count" v-if="search && filteredFunds.length !== funds.length">
          / 筛选后 {{ filteredFunds.length }} 只
        </span>
        <el-input
          v-model="search"
          placeholder="搜索基金代码或名称..."
          clearable
          size="small"
          class="search-input"
        />
      </div>
      <el-button text size="small" @click="openHistoryDir">
        历史缓存
      </el-button>
      <el-button type="primary" @click="fetchAllFunds" :loading="loading" :disabled="loading">
        {{ loading ? progressText : '获取所有基金数据' }}
      </el-button>
      <el-dropdown split-button type="success" size="small" :disabled="historyLoading || !funds.length" @click="fetchAllHistory" @command="batchPeriod = $event">
        {{ historyLoading ? historyProgressText : `批量获取历史数据(${periodLabel})` }}
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item :class="{ active: batchPeriod === '1m' }" command="1m">1个月</el-dropdown-item>
            <el-dropdown-item :class="{ active: batchPeriod === '3m' }" command="3m">3个月</el-dropdown-item>
            <el-dropdown-item :class="{ active: batchPeriod === '6m' }" command="6m">6个月</el-dropdown-item>
            <el-dropdown-item :class="{ active: batchPeriod === '1y' }" command="1y">1年</el-dropdown-item>
            <el-dropdown-item :class="{ active: batchPeriod === 'all' }" command="all">全部</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
    <div class="table-area" ref="tableRef">
      <el-table-v2
        :columns="columns"
        :data="filteredFunds"
        :height="tableHeight"
        :width="tableWidth"
        v-loading="loading"
      />
    </div>
</div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { h } from 'vue'
import { ElButton } from 'element-plus'

interface FundItem {
  id: string
  name: string
  company_id: string
  company_name: string
}

interface CrawlProgress {
  current: number
  total: number
  company_name: string
  status: string
}

interface HistoryCrawlProgress {
  current: number
  total: number
  fund_code: string
  fund_name: string
  status: string
}

let cache = undefined as FundItem[] | undefined

const router = useRouter()
const loading = ref(false)
const progressText = ref('')
const historyLoading = ref(false)
const historyProgressText = ref('')
const batchPeriod = ref('all')
const periodLabel = computed(() => {
  const map: Record<string, string> = { '1m': '1个月', '3m': '3个月', '6m': '6个月', '1y': '1年', all: '全部' }
  return map[batchPeriod.value] || '全部'
})
const search = ref('')

const funds = ref<FundItem[]>([])
const cachedHistoryCodes = ref(new Set<string>())

const filteredFunds = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return funds.value
  return funds.value.filter(f =>
    f.id.toLowerCase().includes(q) || f.name.toLowerCase().includes(q)
  )
})

function hasHistory(code: string) {
  return cachedHistoryCodes.value.has(code)
}

const columns = computed<any[]>(() => [
  { key: 'id', title: '基金代码', width: 110, dataKey: 'id' },
  { key: 'name', title: '基金名称', width: 200, dataKey: 'name',
    cellRenderer: ({ cellData }: any) => cellData },
  { key: 'company_name', title: '基金公司', width: 200, dataKey: 'company_name' },
  { key: 'action', title: '操作', width: 100,
    cellRenderer: ({ rowData }: any) =>
      h(ElButton, { type: 'primary', size: 'small', onClick: () => viewHistory(rowData.id) },
        () => '历史数据'
      )
  },
  {
    key: 'has_history', title: '已获取', width: 70,
    cellRenderer: ({ rowData }: any) =>
      h('span', {
        class: hasHistory(rowData.id) ? 'dot dot-yes' : 'dot dot-no'
      }, hasHistory(rowData.id) ? '是' : '否')
  },
])

const tableRef = ref<HTMLElement | null>(null)
const tableHeight = ref(0)
const tableWidth = ref(0)

function updateTableSize() {
  nextTick(() => {
    if (!tableRef.value) return
    const parent = tableRef.value.parentElement
    if (!parent) return
    const rect = parent.getBoundingClientRect()
    const toolbarHeight = 56
    tableHeight.value = rect.height - toolbarHeight
    tableWidth.value = rect.width
  })
}

async function loadCachedHistoryCodes() {
  try {
    const codes = await invoke<string[]>('get_cached_history_codes')
    cachedHistoryCodes.value = new Set(codes)
  } catch (_e) { }
}

async function fetchAllFunds() {
  loading.value = true
  progressText.value = '0/0'
  let lastUpdate = 0
  const unlisten = await listen<CrawlProgress>('crawl-progress', (event) => {
    const now = Date.now()
    if (now - lastUpdate < 300) return
    lastUpdate = now
    const p = event.payload
    const digits = String(p.total).length
    progressText.value = `${String(p.current).padStart(digits, '0')}/${p.total}`
  })
  try {
    const result = await invoke<FundItem[]>('fetch_all_funds')
    funds.value = result
    cache = result
    progressText.value = `完成，共 ${result.length} 只基金`
    await nextTick()
    updateTableSize()
    loadCachedHistoryCodes()
  } catch (e) {
    console.error(e)
    progressText.value = '获取失败'
  } finally {
    unlisten()
    loading.value = false
  }
}

async function fetchAllHistory() {
  historyLoading.value = true
  historyProgressText.value = '0/0'
  let lastUpdate = 0
  const unlisten = await listen<HistoryCrawlProgress>('history-crawl-progress', (event) => {
    const now = Date.now()
    if (now - lastUpdate < 300) return
    lastUpdate = now
    const p = event.payload
    const digits = String(p.total).length
    historyProgressText.value = `${String(p.current).padStart(digits, '0')}/${p.total}`
  })
  try {
    await invoke('fetch_all_history', { period: batchPeriod.value })
    historyProgressText.value = '全部完成'
    loadCachedHistoryCodes()
  } catch (e) {
    console.error(e)
    historyProgressText.value = '获取失败'
  } finally {
    unlisten()
    historyLoading.value = false
  }
}

async function openHistoryDir() {
  try {
    await invoke('open_history_dir')
  } catch (_e) { }
}

function viewHistory(fundCode: string) {
  router.push(`/fund/${fundCode}`)
}

onMounted(async () => {
  if (cache) {
    funds.value = cache
    await nextTick()
    updateTableSize()
    return
  }
  try {
    const cached = await invoke<FundItem[]>('load_cached_funds')
    if (cached.length > 0) {
      funds.value = cached
      cache = cached
    }
  } catch (_e) { }
  loadCachedHistoryCodes()
  await nextTick()
  updateTableSize()
})

window.addEventListener('resize', updateTableSize)
onBeforeUnmount(() => window.removeEventListener('resize', updateTableSize))
</script>

<style scoped>
.home {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}
.header {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px 0 12px;
  flex-shrink: 0;
}
.header h1 {
  font-size: 36px;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea, #764ba2);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
.subtitle {
  color: #a0aec0;
  margin-top: 4px;
  font-size: 14px;
}
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px 8px;
  flex-shrink: 0;
  gap: 12px;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}
.total-count {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
}
.filtered-count {
  font-size: 13px;
  color: var(--el-color-warning);
  white-space: nowrap;
}
.search-input {
  max-width: 280px;
}
.table-area {
  flex: 1;
  padding: 0 16px 12px;
  overflow: hidden;
}
.table-area :deep(.el-table-v2) {
  --el-table-v2-header-bg-color: rgba(255,255,255,0.06);
  --el-table-v2-row-hover-bg-color: rgba(255,255,255,0.08);
  --el-table-v2-border-color: rgba(255,255,255,0.06);
}
.table-area :deep(.el-table-v2__header-cell) {
  color: #a0aec0;
  font-weight: 600;
}
.table-area :deep(.el-table-v2__row-cell) {
  color: #e0e0e0;
}
.table-area :deep(.el-table-v2__row) {
  background-color: transparent;
}
.dot {
  font-size: 12px;
  font-weight: 600;
}
.dot-yes {
  color: #67c23a;
}
.dot-no {
  color: #f56c6c;
}
</style>

<style>
.el-dropdown-menu__item.active {
  color: var(--el-color-success);
  font-weight: 600;
  background-color: rgba(103, 194, 58, 0.1);
}
</style>
