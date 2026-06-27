<template>
  <div class="simulation">
    <div class="header">
      <el-button text @click="goBack" class="back-btn">← 返回</el-button>
      <div class="fund-title">
        <span class="fund-code">{{ fundCode }}</span>
        <span v-if="fundName" class="fund-name">{{ fundName }}</span>
      </div>
    </div>

    <!-- Phase 1: Data Setup -->
    <template v-if="phase === 'setup'">
      <div class="setup" v-loading="loading" element-loading-text="获取数据中...">
        <div class="setup-row">
          <el-radio-group v-model="period" size="small">
            <el-radio-button value="1m">1个月</el-radio-button>
            <el-radio-button value="3m">3个月</el-radio-button>
            <el-radio-button value="6m">6个月</el-radio-button>
            <el-radio-button value="1y">1年</el-radio-button>
            <el-radio-button value="all">全部</el-radio-button>
          </el-radio-group>
        </div>

        <div v-if="dailyChanges.length" class="range-section">
          <div class="range-header">
            <span class="range-label">选择模拟时间范围</span>
            <span class="range-info">{{ dailyChanges.length }} 个交易日</span>
          </div>
          <div class="date-pickers">
            <el-date-picker
              v-model="rangeStartDate"
              type="date"
              placeholder="开始日期"
              size="small"
              :disabled-date="(d: Date) => d < minDate || d > maxDate"
              value-format="YYYY-MM-DD"
              class="date-picker"
            />
            <span class="range-sep">至</span>
            <el-date-picker
              v-model="rangeEndDate"
              type="date"
              placeholder="结束日期"
              size="small"
              :disabled-date="(d: Date) => d < minDate || d > maxDate"
              value-format="YYYY-MM-DD"
              class="date-picker"
            />
          </div>
          <div class="range-hint" v-if="rangeDayCount > 1">
            共 <strong>{{ rangeDayCount }}</strong> 个交易日可供模拟
          </div>
          <el-button
            type="primary"
            size="small"
            :disabled="rangeDayCount < 2"
            @click="startSimulation"
          >
            开始模拟
          </el-button>
        </div>
      </div>
    </template>

    <!-- Phase 2: Simulation -->
    <template v-if="phase === 'simulate'">
      <div class="sim-body">
        <div class="sim-left">
          <div class="day-header">
            <span class="day-progress">第 {{ currentStep + 1 }} 天 / 共 {{ simDays.length }} 天</span>
            <span class="day-date">{{ currentDay.date }}</span>
          </div>

          <div class="change-card" :class="currentDay.change >= 0 ? 'up' : 'down'">
            <div class="change-label">今日涨跌</div>
            <div class="change-value">{{ currentDay.change >= 0 ? '+' : '' }}{{ currentDay.change.toFixed(2) }}%</div>
          </div>

          <div class="invest-section">
            <div class="invest-label">今日投资额</div>
            <div class="invest-presets">
              <el-button
                v-for="amt in presets"
                :key="amt"
                size="small"
                :class="{ active: currentInvestment === amt }"
                @click="currentInvestment = amt"
              >{{ amt === 0 ? '不投' : `¥${amt.toLocaleString()}` }}</el-button>
            </div>
            <div class="invest-controls">
              <el-input-number
                v-model="currentInvestment"
                :min="0"
                :step="100"
                :precision="2"
                size="small"
                class="invest-input"
                controls-position="right"
              />
              <el-button type="primary" size="small" @click="confirmDay">确认</el-button>
            </div>
          </div>

          <div class="position-bar" v-if="totalInvested > 0">
            <span class="position-label">仓位</span>
            <div class="position-track">
              <div class="position-fill" :style="{ width: positionPct + '%' }"></div>
            </div>
            <span class="position-text">¥{{ formatMoney(totalInvested) }} / ¥{{ formatMoney(totalBudget) }}</span>
          </div>

          </div>

        <div class="sim-right">
          <div class="chart-area" ref="chartRef"></div>
          <div class="summary">
            <div class="summary-row">
              <span>总投资</span>
              <span class="mono">¥{{ formatMoney(totalBudget) }}</span>
            </div>
            <div class="summary-row">
              <span>已投入</span>
              <span class="mono">¥{{ formatMoney(totalInvested) }}</span>
            </div>
            <div class="summary-row">
              <span>可用余额</span>
              <span class="mono" :class="{ warn: totalBudget - totalInvested < 5000 }">¥{{ formatMoney(Math.max(0, totalBudget - totalInvested)) }}</span>
            </div>
            <div class="summary-row">
              <span>累计盈亏</span>
              <span class="mono" :class="cumulativePnl >= 0 ? 'up' : 'down'">
                {{ cumulativePnl >= 0 ? '+' : '' }}¥{{ formatMoney(cumulativePnl) }}
              </span>
            </div>
            <div class="summary-row total">
              <span>当前总价值</span>
              <span class="mono">¥{{ formatMoney(currentValue) }}</span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <el-dialog v-model="showCompleteDialog" title="模拟结束" width="420px" :close-on-click-modal="false" align-center>
      <div class="dialog-stats">
        <div class="dialog-stat">
          <span class="dialog-stat-label">模拟天数</span>
          <span class="dialog-stat-value">{{ simDays.length }} 天</span>
        </div>
        <div class="dialog-stat">
          <span class="dialog-stat-label">总投资额</span>
          <span class="dialog-stat-value mono">¥{{ formatMoney(totalBudget) }}</span>
        </div>
        <div class="dialog-stat">
          <span class="dialog-stat-label">累计投入</span>
          <span class="dialog-stat-value mono">¥{{ formatMoney(totalInvested) }}</span>
        </div>
        <div class="dialog-stat">
          <span class="dialog-stat-label">累计盈亏</span>
          <span class="dialog-stat-value mono" :class="cumulativePnl >= 0 ? 'up' : 'down'">
            {{ cumulativePnl >= 0 ? '+' : '' }}¥{{ formatMoney(cumulativePnl) }}
          </span>
        </div>
        <div class="dialog-stat">
          <span class="dialog-stat-label">最终价值</span>
          <span class="dialog-stat-value mono">¥{{ formatMoney(currentValue) }}</span>
        </div>
        <div class="dialog-stat" v-if="totalInvested > 0">
          <span class="dialog-stat-label">收益率</span>
          <span class="dialog-stat-value" :class="cumulativePnl >= 0 ? 'up' : 'down'">
            {{ ((cumulativePnl / totalInvested) * 100).toFixed(2) }}%
          </span>
        </div>
      </div>
      <template #footer>
        <el-button type="primary" size="small" @click="resetAll">重新开始</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount, nextTick } from "vue"
import { useRoute, useRouter } from "vue-router"
import { invoke } from "@tauri-apps/api/core"
import * as echarts from "echarts"

interface HistoryPoint {
  timestamp: number
  value: number
}

interface FundHistory {
  fund_code: string
  fund_name: string
  data: HistoryPoint[]
}

type Phase = "setup" | "simulate" | "complete"

const route = useRoute()
const router = useRouter()

const fundCode = ref("")
const fundName = ref("")
const period = ref("1m")
const loading = ref(false)
const historyData = ref<HistoryPoint[]>([])
const phase = ref<Phase>("setup")
const showCompleteDialog = ref(false)

function goBack() {
  router.push("/")
}

function formatDate(ts: number) {
  const d = new Date(ts)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`
}

function parseDate(str: string): Date {
  const [y, m, d] = str.split("-").map(Number)
  return new Date(y, m - 1, d)
}

// ─── Daily Changes ───

const dailyChanges = computed(() => {
  const sorted = [...historyData.value].sort((a, b) => a.timestamp - b.timestamp)
  const changes: { date: string; change: number }[] = []
  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1].value
    const cur = sorted[i].value
    const change = ((cur - prev) / (100 + prev)) * 100
    changes.push({ date: formatDate(sorted[i].timestamp), change })
  }
  return changes
})

const minDate = computed(() => {
  if (!dailyChanges.value.length) return new Date()
  return parseDate(dailyChanges.value[0].date)
})

const maxDate = computed(() => {
  if (!dailyChanges.value.length) return new Date()
  return parseDate(dailyChanges.value[dailyChanges.value.length - 1].date)
})

// ─── Range Selection ───

const rangeStartDate = ref<string>("")
const rangeEndDate = ref<string>("")

function findLastIndex<T>(arr: T[], fn: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (fn(arr[i])) return i
  }
  return -1
}

const rangeDayCount = computed(() => {
  if (!rangeStartDate.value || !rangeEndDate.value) return 0
  const startIdx = dailyChanges.value.findIndex(d => d.date >= rangeStartDate.value)
  const endIdx = findLastIndex(dailyChanges.value, d => d.date <= rangeEndDate.value)
  if (startIdx === -1 || endIdx === -1 || endIdx <= startIdx) return 0
  return endIdx - startIdx + 1
})

// ─── Simulation State ───

interface SimulationDay {
  date: string
  change: number
  investment: number
  pnl: number
  position: number
}

const simDays = ref<SimulationDay[]>([])
const currentStep = ref(0)
const currentInvestment = ref(1000)
const presets = [0, 100, 500, 1000, 5000, 10000]
const currentDay = computed(() => simDays.value[currentStep.value] ?? { date: "", change: 0, investment: 0, pnl: 0, position: 0 })

const totalInvested = computed(() =>
  simDays.value.reduce((sum, d) => sum + d.investment, 0)
)

const cumulativePnl = computed(() =>
  simDays.value.reduce((sum, d) => sum + d.pnl, 0)
)

const totalBudget = 50000
const positionPct = computed(() =>
  Math.min((totalInvested.value / totalBudget) * 100, 100)
)
const currentValue = computed(() => totalInvested.value + cumulativePnl.value)

function formatMoney(n: number): string {
  return n.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

// ─── Chart ───

const chartRef = ref<HTMLElement | null>(null)
let chartInstance: echarts.ECharts | null = null

const chartPoints = computed(() => {
  if (!simDays.value.length || !historyData.value.length) return []
  const sorted = [...historyData.value].sort((a, b) => a.timestamp - b.timestamp)
  const firstSimDate = simDays.value[0].date
  const firstPointIdx = sorted.findIndex(p => formatDate(p.timestamp) === firstSimDate)
  if (firstPointIdx < 0) return sorted.slice(0, 2)
  const showUpTo = Math.min(firstPointIdx + currentStep.value, sorted.length - 1)
  const result = sorted.slice(0, 1)
  for (let i = firstPointIdx; i <= showUpTo; i++) {
    result.push(sorted[i])
  }
  return result
})

function buildChartOption() {
  const points = chartPoints.value
  if (!points.length) return {}

  const currentTs = currentStep.value < simDays.value.length
    ? parseDate(simDays.value[currentStep.value].date).getTime()
    : points[points.length - 1].timestamp

  const markLines: any[] = [{
    xAxis: currentTs,
    label: { show: false },
    lineStyle: { color: "rgba(255,255,255,0.25)", type: "dashed", width: 1 },
    symbol: "none",
  }]

  // Add investment markPoints
  const markPoints: any[] = []
  simDays.value.forEach((day) => {
    const pt = points.find(p => formatDate(p.timestamp) === day.date)
    if (!pt) return
    const isZero = day.investment === 0
    markPoints.push({
      coord: [pt.timestamp, pt.value],
      symbol: "circle",
      symbolSize: 8,
      itemStyle: {
        color: "#D4A84B",
        borderColor: "#0B0B0F",
        borderWidth: 2,
      },
      label: {
        show: true,
        formatter: isZero ? "0" : `¥${day.investment >= 10000 ? (day.investment / 10000).toFixed(1) + 'w' : day.investment.toFixed(0)}`,
        position: "top",
        color: "#D4A84B",
        fontSize: 10,
        fontFamily: "SF Mono, monospace",
      },
    })
  })

  return {
    backgroundColor: "transparent",
    grid: { left: 50, right: 16, top: 28, bottom: 28 },
    tooltip: {
      trigger: "axis",
      formatter: (params: any) => {
        const p = params[0]
        if (!p) return ""
        return `${formatDate(p.data[0])}<br/>累计收益率: ${p.data[1].toFixed(2)}%`
      },
    },
    xAxis: {
      type: "time",
      axisLabel: { color: "#5A5A5A", fontSize: 10, hideOverlap: true },
      splitLine: { show: false },
      axisLine: { show: false },
      axisTick: { show: false },
    },
    yAxis: {
      type: "value",
      axisLabel: {
        color: "#5A5A5A",
        fontSize: 10,
        formatter: (v: number) => v.toFixed(1) + "%",
      },
      splitLine: { lineStyle: { color: "rgba(255,255,255,0.04)" } },
      axisLine: { show: false },
      axisTick: { show: false },
    },
    series: [{
      type: "line",
      data: points.map(p => [p.timestamp, p.value]),
      smooth: true,
      showSymbol: false,
      lineStyle: { width: 2, color: "#D4A84B" },
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: "rgba(212,168,75,0.2)" },
          { offset: 1, color: "rgba(212,168,75,0.01)" },
        ]),
      },
      markLine: {
        silent: true,
        symbol: "none",
        data: markLines,
      },
      markPoint: {
        silent: true,
        symbol: "circle",
        data: markPoints,
      },
    }],
  }
}

function renderChart() {
  if (!chartRef.value || !chartPoints.value.length) return
  if (!chartInstance) {
    chartInstance = echarts.init(chartRef.value, undefined, { renderer: "canvas" })
  }
  chartInstance.setOption(buildChartOption(), true)
  chartInstance.resize()
}

function updateChartMarker() {
  if (!chartInstance || !chartPoints.value.length) return
  chartInstance.setOption(buildChartOption(), true)
}

// ─── Watch chart ───

watch(currentStep, () => {
  nextTick(() => updateChartMarker())
})

watch(simDays, () => {
  nextTick(() => updateChartMarker())
}, { deep: true })

watch(period, () => {
  if (fundCode.value && phase.value === "setup") {
    fetchData()
  }
})

// ─── Actions ───

async function fetchData() {
  loading.value = true
  try {
    const result = await invoke<FundHistory>("get_fund_history", {
      fundCode: fundCode.value,
      period: period.value,
    })
    fundName.value = result.fund_name
    historyData.value = result.data
    // auto-set range to full available
    if (dailyChanges.value.length) {
      rangeStartDate.value = dailyChanges.value[0].date
      rangeEndDate.value = dailyChanges.value[dailyChanges.value.length - 1].date
    }
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function startSimulation() {
  const startIdx = dailyChanges.value.findIndex(d => d.date >= rangeStartDate.value)
  const endIdx = findLastIndex(dailyChanges.value, d => d.date <= rangeEndDate.value)
  if (startIdx === -1 || endIdx === -1 || endIdx <= startIdx) return

  simDays.value = dailyChanges.value.slice(startIdx, endIdx + 1).map(d => ({
    date: d.date,
    change: d.change,
    investment: 0,
    pnl: 0,
    position: 0,
  }))
  currentStep.value = 0
  currentInvestment.value = 1000
  phase.value = "simulate"
  nextTick(() => renderChart())
}

function confirmDay() {
  const step = currentStep.value
  const remaining = totalBudget - totalInvested.value
  let inv = currentInvestment.value || 0
  if (inv > remaining) inv = remaining
  currentInvestment.value = inv

  const day = simDays.value[step]
  day.investment = inv

  if (step === 0) {
    day.pnl = 0
    day.position = inv
  } else {
    const prevPosition = simDays.value[step - 1].position
    day.pnl = prevPosition * (day.change / 100)
    day.position = prevPosition + day.pnl + inv
  }

  if (step >= simDays.value.length - 1) {
    showCompleteDialog.value = true
  } else {
    currentStep.value++
  }
}

function resetAll() {
  phase.value = "setup"
  showCompleteDialog.value = false
  simDays.value = []
  currentStep.value = 0
  // reset chart
  chartInstance?.dispose()
  chartInstance = null
}

// ─── Lifecycle ───

onMounted(() => {
  fundCode.value = route.params.code as string
  window.addEventListener("resize", handleResize)
  if (fundCode.value) {
    fetchData()
  }
})

onBeforeUnmount(() => {
  window.removeEventListener("resize", handleResize)
  chartInstance?.dispose()
  chartInstance = null
})

function handleResize() {
  chartInstance?.resize()
}

watch(
  () => route.params.code,
  (code) => {
    if (code && code !== fundCode.value) {
      fundCode.value = code as string
      fundName.value = ""
      resetAll()
    }
  },
)
</script>

<style scoped>
.simulation {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}
.header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-subtle);
}
.back-btn {
  font-size: 13px;
  flex-shrink: 0;
}
.fund-title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex: 1;
  min-width: 0;
}
.fund-code {
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 700;
  color: var(--accent-gold);
  letter-spacing: 0.02em;
  flex-shrink: 0;
}
.fund-name {
  font-size: 15px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ─── Setup ─── */

.setup {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  padding: 40px 16px;
}
.setup-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.range-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}
.range-header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.range-label {
  font-size: 14px;
  color: var(--text-primary);
}
.range-info {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: var(--font-display);
}
.date-pickers {
  display: flex;
  align-items: center;
  gap: 8px;
}
.date-picker {
  width: 160px;
}
.range-sep {
  color: var(--text-muted);
  font-size: 13px;
}
.range-hint {
  font-size: 13px;
  color: var(--text-secondary);
}
.range-hint strong {
  color: var(--accent-gold);
  font-family: var(--font-display);
}

/* ─── Simulation ─── */

.sim-body {
  flex: 1;
  display: flex;
  gap: 0;
  overflow: hidden;
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
}

.sim-left {
  flex: 1;
  overflow-y: auto;
  padding: 20px 16px 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.sim-right {
  width: 380px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px 20px 16px 8px;
  border-left: 1px solid var(--border-subtle);
}

.chart-area {
  flex: 1;
  min-height: 0;
  width: 100%;
}
.day-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.day-progress {
  font-size: 13px;
  color: var(--text-secondary);
  font-family: var(--font-display);
}
.day-date {
  font-size: 13px;
  color: var(--text-muted);
  font-family: var(--font-display);
}

/* Change Card */
.change-card {
  text-align: center;
  padding: 24px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  background: var(--bg-surface);
}
.change-card.up {
  border-color: rgba(231, 76, 76, 0.25);
  background: rgba(231, 76, 76, 0.05);
}
.change-card.down {
  border-color: rgba(39, 174, 96, 0.25);
  background: rgba(39, 174, 96, 0.05);
}
.change-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}
.change-value {
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: 700;
  letter-spacing: -0.02em;
}
.change-card.up .change-value {
  color: var(--up-red);
}
.change-card.down .change-value {
  color: var(--down-green);
}

/* P&L Section */
.pnl-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: var(--bg-surface);
  border-radius: var(--radius-sm);
}
.pnl-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.pnl-label {
  font-size: 13px;
  color: var(--text-secondary);
}
.pnl-value {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 600;
}
.pnl-value.up {
  color: var(--up-red);
}
.pnl-value.down {
  color: var(--down-green);
}

/* Investment */
.invest-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.invest-label {
  font-size: 13px;
  color: var(--text-primary);
}
.invest-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.invest-presets .el-button {
  font-family: var(--font-display);
  font-size: 12px;
  padding: 0 10px;
  height: 28px;
  border-color: var(--border-default);
  color: var(--text-secondary);
  background: transparent;
  border-radius: var(--radius-sm);
}
.invest-presets .el-button:hover {
  border-color: var(--accent-gold);
  color: var(--accent-gold);
}
.invest-presets .el-button.active {
  border-color: var(--accent-gold);
  background: var(--accent-gold-muted);
  color: var(--accent-gold);
}
.invest-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}
.invest-input {
  flex: 1;
}
.invest-input :deep(.el-input__inner) {
  font-family: var(--font-display);
  text-align: right;
}

/* Summary */
.summary {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  background: var(--bg-surface);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
}
.summary-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: var(--text-secondary);
}
.summary-row.total {
  border-top: 1px solid var(--border-subtle);
  padding-top: 8px;
  margin-top: 2px;
  font-weight: 600;
  color: var(--text-primary);
}
.summary-row .mono {
  font-family: var(--font-display);
}
.summary-row .up {
  color: var(--up-red);
}
.summary-row .down {
  color: var(--down-green);
}
.summary-row .warn {
  color: var(--accent-gold);
}

/* ─── Position Bar ─── */

.position-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-surface);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
}
.position-label {
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}
.position-track {
  flex: 1;
  height: 4px;
  background: rgba(255,255,255,0.06);
  border-radius: 2px;
  overflow: hidden;
}
.position-fill {
  height: 100%;
  background: var(--accent-gold);
  border-radius: 2px;
  transition: width 0.3s ease;
}
.position-text {
  font-family: var(--font-display);
  font-size: 11px;
  color: var(--text-muted);
  flex-shrink: 0;
}

/* Navigation */
.nav-buttons {
  display: flex;
  justify-content: space-between;
  padding-top: 4px;
}

/* ─── Dialog ─── */

.dialog-stats {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.dialog-stat {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  background: var(--bg-surface);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
}
.dialog-stat-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.dialog-stat-value {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}
.dialog-stat-value.up {
  color: var(--up-red);
}
.dialog-stat-value.down {
  color: var(--down-green);
}

:deep(.el-dialog) {
  --el-dialog-bg-color: var(--bg-elevated);
  --el-dialog-title-font-size: 16px;
  --el-dialog-border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
}
:deep(.el-dialog__title) {
  color: var(--text-primary);
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 16px;
}
:deep(.el-dialog__header) {
  border-bottom: 1px solid var(--border-subtle);
  padding: 16px 20px;
  margin: 0;
}
:deep(.el-dialog__body) {
  padding: 20px;
}
:deep(.el-dialog__footer) {
  border-top: 1px solid var(--border-subtle);
  padding: 12px 20px;
}


</style>
