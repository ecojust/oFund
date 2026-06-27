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
      <div class="setup">
        <div class="setup-row">
          <el-radio-group v-model="period" size="small">
            <el-radio-button value="1m">1个月</el-radio-button>
            <el-radio-button value="3m">3个月</el-radio-button>
            <el-radio-button value="6m">6个月</el-radio-button>
            <el-radio-button value="1y">1年</el-radio-button>
            <el-radio-button value="all">全部</el-radio-button>
          </el-radio-group>
          <el-button type="primary" size="small" :loading="loading" @click="fetchData">
            {{ historyData.length ? '重新获取' : '获取数据' }}
          </el-button>
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
        <div class="day-header">
          <span class="day-progress">第 {{ currentStep + 1 }} 天 / 共 {{ simDays.length }} 天</span>
          <span class="day-date">{{ currentDay.date }}</span>
        </div>

        <div class="change-card" :class="currentDay.change >= 0 ? 'up' : 'down'">
          <div class="change-label">今日涨跌</div>
          <div class="change-value">{{ currentDay.change >= 0 ? '+' : '' }}{{ currentDay.change.toFixed(2) }}%</div>
        </div>

        <div class="pnl-section" v-if="currentStep > 0">
          <div class="pnl-row">
            <span class="pnl-label">上期投入</span>
            <span class="pnl-value">¥{{ formatMoney(prevInvestment) }}</span>
          </div>
          <div class="pnl-row">
            <span class="pnl-label">今日盈亏</span>
            <span class="pnl-value" :class="todayPnl >= 0 ? 'up' : 'down'">
              {{ todayPnl >= 0 ? '+' : '' }}¥{{ formatMoney(todayPnl) }}
            </span>
          </div>
        </div>

        <div class="invest-section">
          <div class="invest-label">今日投资额</div>
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

        <div class="summary">
          <div class="summary-row">
            <span>累计投入</span>
            <span class="mono">¥{{ formatMoney(totalInvested) }}</span>
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

        <div class="nav-buttons" v-if="allConfirmed.length">
          <el-button text size="small" :disabled="currentStep === 0" @click="goToStep(currentStep - 1)">
            ← 上一天
          </el-button>
          <el-button text size="small" :disabled="currentStep >= simDays.length - 1" @click="goToStep(currentStep + 1)">
            下一天 →
          </el-button>
        </div>
      </div>
    </template>

    <!-- Phase 3: Complete -->
    <template v-if="phase === 'complete'">
      <div class="complete">
        <div class="complete-summary">
          <div class="complete-title">模拟结束</div>
          <div class="complete-stats">
            <div class="stat">
              <span class="stat-label">模拟天数</span>
              <span class="stat-value">{{ simDays.length }} 天</span>
            </div>
            <div class="stat">
              <span class="stat-label">累计投入</span>
              <span class="stat-value mono">¥{{ formatMoney(totalInvested) }}</span>
            </div>
            <div class="stat">
              <span class="stat-label">累计盈亏</span>
              <span class="stat-value mono" :class="cumulativePnl >= 0 ? 'up' : 'down'">
                {{ cumulativePnl >= 0 ? '+' : '' }}¥{{ formatMoney(cumulativePnl) }}
              </span>
            </div>
            <div class="stat">
              <span class="stat-label">最终价值</span>
              <span class="stat-value mono">¥{{ formatMoney(currentValue) }}</span>
            </div>
            <div class="stat" v-if="totalInvested > 0">
              <span class="stat-label">收益率</span>
              <span class="stat-value" :class="cumulativePnl >= 0 ? 'up' : 'down'">
                {{ ((cumulativePnl / totalInvested) * 100).toFixed(2) }}%
              </span>
            </div>
          </div>
          <el-button text size="small" @click="goToStep(0)" class="review-btn">回顾模拟过程</el-button>
          <el-button text size="small" @click="resetAll" class="reset-btn">重新开始</el-button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { invoke } from "@tauri-apps/api/core"

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
}

const simDays = ref<SimulationDay[]>([])
const currentStep = ref(0)
const currentInvestment = ref(1000)
const allConfirmed = ref<number[]>([]) // indices that have been confirmed

const currentDay = computed(() => simDays.value[currentStep.value] ?? { date: "", change: 0, investment: 0, pnl: 0 })

const prevInvestment = computed(() => {
  if (currentStep.value === 0) return 0
  return simDays.value[currentStep.value - 1].investment
})

const todayPnl = computed(() => {
  if (currentStep.value === 0) return 0
  const prevInv = simDays.value[currentStep.value - 1].investment
  return prevInv * (currentDay.value.change / 100)
})

const totalInvested = computed(() =>
  simDays.value.reduce((sum, d) => sum + d.investment, 0)
)

const cumulativePnl = computed(() =>
  simDays.value.reduce((sum, d) => sum + d.pnl, 0)
)

const currentValue = computed(() => totalInvested.value + cumulativePnl.value)

function formatMoney(n: number): string {
  return n.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

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
  }))
  currentStep.value = 0
  currentInvestment.value = 1000
  allConfirmed.value = []
  phase.value = "simulate"
}

function confirmDay() {
  const inv = currentInvestment.value || 0
  simDays.value[currentStep.value].investment = inv
  if (currentStep.value > 0) {
    const prevInv = simDays.value[currentStep.value - 1].investment
    simDays.value[currentStep.value].pnl = prevInv * (simDays.value[currentStep.value].change / 100)
  }
  allConfirmed.value.push(currentStep.value)

  if (currentStep.value >= simDays.value.length - 1) {
    // calc last day's pnl
    const lastInv = simDays.value[simDays.value.length - 1].investment
    if (lastInv > 0) {
      // no next day change to apply, so no pnl from last investment
    }
    phase.value = "complete"
  } else {
    currentStep.value++
    currentInvestment.value = 1000
  }
}

function goToStep(idx: number) {
  if (idx >= 0 && idx < simDays.value.length) {
    currentStep.value = idx
  }
}

function resetAll() {
  phase.value = "setup"
  simDays.value = []
  currentStep.value = 0
  allConfirmed.value = []
  historyData.value = []
}

// ─── Lifecycle ───

onMounted(() => {
  fundCode.value = route.params.code as string
})

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
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 480px;
  margin: 0 auto;
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

/* Navigation */
.nav-buttons {
  display: flex;
  justify-content: space-between;
  padding-top: 4px;
}

/* ─── Complete ─── */

.complete {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 16px;
}
.complete-summary {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}
.complete-title {
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
}
.complete-stats {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  text-align: center;
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px;
  background: var(--bg-surface);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
}
.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.stat-value {
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}
.stat-value.up {
  color: var(--up-red);
}
.stat-value.down {
  color: var(--down-green);
}
.review-btn,
.reset-btn {
  font-size: 13px;
}
</style>
