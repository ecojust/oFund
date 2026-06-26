<template>
  <div class="detail">
    <div class="header">
      <el-button text @click="goBack" class="back-btn">← 返回</el-button>
      <h2>{{ fundCode }}{{ fundName ? ' - ' + fundName : '' }}</h2>
      <el-radio-group v-if="fundCode" v-model="period" size="small" @change="fetchHistory">
        <el-radio-button value="1m">1个月</el-radio-button>
        <el-radio-button value="3m">3个月</el-radio-button>
        <el-radio-button value="6m">6个月</el-radio-button>
        <el-radio-button value="1y">1年</el-radio-button>
        <el-radio-button value="all">成立以来</el-radio-button>
      </el-radio-group>
      <el-button v-if="fundCode" type="primary" @click="fetchHistory" :loading="loading" size="small">
        {{ history.length ? '刷新数据' : '获取历史数据' }}
      </el-button>
    </div>

    <div class="content" v-loading="loading">
      <div v-if="history.length" class="chart-area">
        <div ref="chartRef" class="chart"></div>
      </div>

      <div class="calendar-area" v-if="history.length">
        <div class="calendar-header">
          <h3>{{ calendarDate.getFullYear() }}年{{ calendarDate.getMonth() + 1 }}月</h3>
        </div>
        <el-calendar v-model="calendarDate">
          <template #date-cell="{ data }">
            <div class="calendar-cell" :class="getCellClass(data.day)">
              <span class="calendar-day">{{ data.day.split('-').pop() }}</span>
              <span v-if="getDailyReturn(data.day) !== null" class="calendar-return" :class="getReturnClass(getDailyReturn(data.day)!)">
                {{ getDailyReturn(data.day)!.toFixed(2) }}%
              </span>
            </div>
          </template>
        </el-calendar>
      </div>

      <div v-if="!history.length && !loading" class="empty">
        <el-empty description="暂无数据，请点击上方按钮获取" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import * as echarts from 'echarts'

interface HistoryPoint {
  timestamp: number
  value: number
}

interface FundHistory {
  fund_code: string
  fund_name: string
  data: HistoryPoint[]
}

const route = useRoute()
const router = useRouter()
const fundCode = ref('')
const fundName = ref('')
const period = ref('all')
const history = ref<HistoryPoint[]>([])
const loading = ref(false)
const chartRef = ref<HTMLElement | null>(null)
const calendarDate = ref(new Date())

const dailyReturnMap = computed(() => {
  const sorted = [...history.value].sort((a, b) => a.timestamp - b.timestamp)
  const map = new Map<string, number>()
  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1].value
    const cur = sorted[i].value
    const dailyReturn = (cur - prev) / (100 + prev) * 100
    map.set(formatDate(sorted[i].timestamp), dailyReturn)
  }
  return map
})

function goBack() {
  router.push('/')
}

function formatDate(ts: number) {
  const d = new Date(ts)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

function getDailyReturn(dateStr: string): number | null {
  return dailyReturnMap.value.get(dateStr) ?? null
}

function getCellClass(dateStr: string) {
  const ret = getDailyReturn(dateStr)
  if (ret === null) return ''
  return ret >= 0 ? 'positive' : 'negative'
}

function getReturnClass(ret: number) {
  return ret >= 0 ? 'up' : 'down'
}

async function fetchHistory() {
  loading.value = true
  try {
    const result = await invoke<FundHistory>('get_fund_history', { fundCode: fundCode.value, period: period.value })
    fundName.value = result.fund_name
    history.value = result.data
    await nextTick()
    renderChart()
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function renderChart() {
  if (!chartRef.value || !history.value.length) return
  const chart = echarts.init(chartRef.value)
  const sorted = [...history.value].sort((a, b) => a.timestamp - b.timestamp)
  chart.setOption({
    backgroundColor: 'transparent',
    grid: { left: 60, right: 20, top: 30, bottom: 40 },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const p = params[0]
        return `${formatDate(p.data[0])}<br/>累计收益率: ${p.data[1].toFixed(2)}%`
      }
    },
    xAxis: {
      type: 'time',
      axisLabel: { color: '#a0aec0', fontSize: 11 },
      splitLine: { show: false },
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: '#a0aec0',
        fontSize: 11,
        formatter: (v: number) => v.toFixed(1) + '%'
      },
      splitLine: { lineStyle: { color: 'rgba(255,255,255,0.06)' } },
    },
    series: [{
      type: 'line',
      data: sorted.map(p => [p.timestamp, p.value]),
      smooth: true,
      showSymbol: false,
      lineStyle: { width: 2, color: '#667eea' },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: 'rgba(102,126,234,0.35)' },
          { offset: 1, color: 'rgba(102,126,234,0.02)' }
        ])
      }
    }]
  })
  chart.resize()
}

onMounted(() => {
  fundCode.value = route.params.code as string
  if (fundCode.value) {
    fetchHistory()
  }
})

watch(() => route.params.code, (code) => {
  if (code && code !== fundCode.value) {
    fundCode.value = code as string
    history.value = []
    fundName.value = ''
    fetchHistory()
  }
})
</script>

<style scoped>
.detail {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}
.header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  flex-shrink: 0;
}
.header h2 {
  font-size: 18px;
  font-weight: 600;
  flex: 1;
}
.detail .content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.back-btn {
  font-size: 14px;
}
.chart-area {
  height: 300px;
  flex-shrink: 0;
  padding: 0 16px 12px;
}
.chart {
  width: 100%;
  height: 100%;
}
.calendar-area {
  flex: 1;
  overflow: auto;
  padding: 0 16px 12px;
}
.calendar-header {
  padding: 8px 0;
}
.calendar-header h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.calendar-area :deep(.el-calendar) {
  --el-calendar-border: var(--el-border-color);
}
.calendar-area :deep(.el-calendar-table td.is-selected) {
  background-color: transparent;
}
.calendar-area :deep(.el-calendar-table td.is-today) {
  background-color: rgba(102, 126, 234, 0.08);
}
.calendar-area :deep(.el-calendar-day) {
  height: auto;
  padding: 4px;
}
.calendar-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  min-height: 48px;
}
.calendar-day {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.calendar-return {
  font-size: 11px;
  font-weight: 600;
}
.calendar-return.up {
  color: #f56c6c;
}
.calendar-return.down {
  color: #67c23a;
}
.empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
