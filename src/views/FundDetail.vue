<template>
  <div class="detail">
    <div class="header">
      <button class="back-btn" @click="goBack">← 返回</button>
      <div class="fund-title">
        <span class="fund-code">{{ fundCode }}</span>
        <span v-if="fundName" class="fund-name">{{ fundName }}</span>
      </div>
      <div v-if="fundCode" class="period-group">
        <button
          v-for="opt in periodOptions"
          :key="opt.value"
          class="period-btn"
          :class="{ active: period === opt.value }"
          @click="selectPeriod(opt.value)"
        >
          {{ opt.label }}
        </button>
      </div>
      <button
        v-if="fundCode"
        class="btn-primary"
        :disabled="loading"
        @click="fetchHistory"
      >
        <span v-if="loading" class="spinner"></span>
        {{ history.length ? "刷新" : "获取数据" }}
      </button>
    </div>

    <div class="content">
      <div v-if="loading" class="loading-overlay">
        <div class="loading-spinner"></div>
      </div>

      <div v-if="history.length" class="chart-area">
        <div ref="chartRef" class="chart"></div>
      </div>

      <div v-if="history.length" class="calendar-area">
        <div class="calendar-toolbar">
          <button class="cal-nav" @click="prevMonth">‹</button>
          <h3>{{ calendarDate.getFullYear() }}年{{ calendarDate.getMonth() + 1 }}月</h3>
          <button class="cal-nav" @click="nextMonth">›</button>
        </div>
        <table class="calendar-table">
          <thead>
            <tr>
              <th v-for="d in weekDays" :key="d">{{ d }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(week, wi) in calendarDays" :key="wi">
              <td
                v-for="(cell, ci) in week"
                :key="ci"
                :class="{ 'is-empty': cell.isEmpty, 'is-today': cell.dateStr === todayStr }"
              >
                <div v-if="!cell.isEmpty" class="calendar-cell" :class="getCellClass(cell.dateStr)">
                  <span class="cal-day-num">{{ cell.day }}</span>
                  <span
                    v-if="getDailyReturn(cell.dateStr) !== null"
                    class="cal-return"
                    :class="getReturnClass(getDailyReturn(cell.dateStr)!)"
                  >
                    {{ getDailyReturn(cell.dateStr)!.toFixed(2) }}%
                  </span>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="!history.length && !loading" class="empty">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <path d="M9 3v18M15 3v18M3 9h18M3 15h18"/>
        </svg>
        <p>暂无数据</p>
        <p class="empty-hint">请点击上方按钮获取</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import * as echarts from "echarts";

interface HistoryPoint {
  timestamp: number;
  value: number;
}

interface FundHistory {
  fund_code: string;
  fund_name: string;
  data: HistoryPoint[];
}

const route = useRoute();
const router = useRouter();
const fundCode = ref("");
const fundName = ref("");
const period = ref("1m");
const history = ref<HistoryPoint[]>([]);
const loading = ref(false);
const chartRef = ref<HTMLElement | null>(null);
const calendarDate = ref(new Date());

const periodOptions = [
  { value: "1m", label: "1个月" },
  { value: "3m", label: "3个月" },
  { value: "6m", label: "6个月" },
  { value: "1y", label: "1年" },
  { value: "all", label: "成立以来" },
];

const weekDays = ["日", "一", "二", "三", "四", "五", "六"];

const dailyReturnMap = computed(() => {
  const sorted = [...history.value].sort((a, b) => a.timestamp - b.timestamp);
  const map = new Map<string, number>();
  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1].value;
    const cur = sorted[i].value;
    const dailyReturn = ((cur - prev) / (100 + prev)) * 100;
    map.set(formatDate(sorted[i].timestamp), dailyReturn);
  }
  return map;
});

const todayStr = computed(() => {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
});

const calendarDays = computed(() => {
  const year = calendarDate.value.getFullYear();
  const month = calendarDate.value.getMonth();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const firstDay = new Date(year, month, 1).getDay();
  const weeks: Array<Array<{ day: number; dateStr: string; isEmpty: boolean }>> = [];
  let week: Array<{ day: number; dateStr: string; isEmpty: boolean }> = [];

  for (let i = 0; i < firstDay; i++) {
    week.push({ day: 0, dateStr: "", isEmpty: true });
  }
  for (let d = 1; d <= daysInMonth; d++) {
    const dateStr = `${year}-${String(month + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    week.push({ day: d, dateStr, isEmpty: false });
    if (week.length === 7) {
      weeks.push(week);
      week = [];
    }
  }
  if (week.length) {
    while (week.length < 7) week.push({ day: 0, dateStr: "", isEmpty: true });
    weeks.push(week);
  }
  return weeks;
});

function prevMonth() {
  const d = new Date(calendarDate.value);
  d.setMonth(d.getMonth() - 1);
  calendarDate.value = d;
}

function nextMonth() {
  const d = new Date(calendarDate.value);
  d.setMonth(d.getMonth() + 1);
  calendarDate.value = d;
}

function goBack() {
  router.push("/");
}

function formatDate(ts: number) {
  const d = new Date(ts);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function getDailyReturn(dateStr: string): number | null {
  return dailyReturnMap.value.get(dateStr) ?? null;
}

function getCellClass(dateStr: string) {
  const ret = getDailyReturn(dateStr);
  if (ret === null) return "";
  return ret >= 0 ? "positive" : "negative";
}

function getReturnClass(ret: number) {
  return ret >= 0 ? "up" : "down";
}

function selectPeriod(value: string) {
  period.value = value;
  fetchHistory();
}

async function fetchHistory() {
  loading.value = true;
  try {
    const result = await invoke<FundHistory>("get_fund_history", {
      fundCode: fundCode.value,
      period: period.value,
    });
    fundName.value = result.fund_name;
    history.value = result.data;
    await nextTick();
    renderChart();
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

function renderChart() {
  if (!chartRef.value || !history.value.length) return;
  const chart = echarts.init(chartRef.value);
  const sorted = [...history.value].sort((a, b) => a.timestamp - b.timestamp);
  chart.setOption({
    backgroundColor: "transparent",
    grid: { left: 60, right: 20, top: 30, bottom: 40 },
    tooltip: {
      trigger: "axis",
      formatter: (params: any) => {
        const p = params[0];
        return `${formatDate(p.data[0])}<br/>累计收益率: ${p.data[1].toFixed(2)}%`;
      },
    },
    xAxis: {
      type: "time",
      axisLabel: { color: "#a0aec0", fontSize: 11 },
      splitLine: { show: false },
    },
    yAxis: {
      type: "value",
      axisLabel: {
        color: "#a0aec0",
        fontSize: 11,
        formatter: (v: number) => v.toFixed(1) + "%",
      },
      splitLine: { lineStyle: { color: "rgba(255,255,255,0.06)" } },
    },
    series: [
      {
        type: "line",
        data: sorted.map((p) => [p.timestamp, p.value]),
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 2, color: "#D4A84B" },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: "rgba(212,168,75,0.25)" },
            { offset: 1, color: "rgba(212,168,75,0.01)" },
          ]),
        },
      },
    ],
  });
  chart.resize();
}

onMounted(() => {
  fundCode.value = route.params.code as string;
  if (fundCode.value) {
    fetchHistory();
  }
});

watch(
  () => route.params.code,
  (code) => {
    if (code && code !== fundCode.value) {
      fundCode.value = code as string;
      history.value = [];
      fundName.value = "";
      fetchHistory();
    }
  },
);
</script>

<style scoped>
.detail {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* ─── Header ─── */

.header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-subtle);
}
.back-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-family: var(--font-body);
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  transition: all 0.15s ease;
  flex-shrink: 0;
}
.back-btn:hover {
  color: var(--accent-gold);
  background: var(--accent-gold-muted);
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

/* ─── Period Group ─── */

.period-group {
  display: flex;
  flex-shrink: 0;
}
.period-btn {
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border-default);
  background: transparent;
  color: var(--text-secondary);
  font-family: var(--font-body);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  margin-left: -1px;
}
.period-btn:first-child {
  margin-left: 0;
  border-radius: var(--radius-sm) 0 0 var(--radius-sm);
}
.period-btn:last-child {
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
}
.period-btn:hover {
  background: var(--accent-gold-muted);
  color: var(--accent-gold);
  border-color: var(--accent-gold);
  z-index: 1;
  position: relative;
}
.period-btn.active {
  background: var(--accent-gold);
  border-color: var(--accent-gold);
  color: #0B0B0F;
  z-index: 1;
  position: relative;
}

/* ─── Primary Button ─── */

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 12px;
  border-radius: var(--radius-sm);
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s ease;
  border: 1px solid var(--accent-gold);
  background: var(--accent-gold);
  color: #0B0B0F;
  line-height: 1;
}
.btn-primary:hover:not(:disabled) {
  background: var(--accent-gold-hover);
  border-color: var(--accent-gold-hover);
}
.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ─── Spinner ─── */

.spinner {
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  flex-shrink: 0;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ─── Content ─── */

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

/* ─── Loading Overlay ─── */

.loading-overlay {
  position: absolute;
  inset: 0;
  background: rgba(11, 11, 15, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  backdrop-filter: blur(2px);
}
.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--accent-gold-muted);
  border-top-color: var(--accent-gold);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ─── Chart ─── */

.chart-area {
  height: 300px;
  flex-shrink: 0;
  padding: 16px 16px 8px;
}
.chart {
  width: 100%;
  height: 100%;
}

/* ─── Calendar ─── */

.calendar-area {
  flex: 1;
  overflow: auto;
  padding: 0 16px 12px;
}
.calendar-toolbar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 8px 0 12px;
}
.calendar-toolbar h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  font-family: var(--font-display);
  min-width: 110px;
  text-align: center;
}
.cal-nav {
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  transition: all 0.15s ease;
}
.cal-nav:hover {
  color: var(--accent-gold);
  border-color: var(--accent-gold);
  background: var(--accent-gold-muted);
}

/* Calendar Table */
.calendar-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}
.calendar-table th {
  padding: 6px 0;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-align: center;
  border-bottom: 1px solid var(--border-subtle);
}
.calendar-table td {
  border: 1px solid var(--border-subtle);
  text-align: center;
  vertical-align: top;
  padding: 0;
  height: 62px;
}
.calendar-table td.is-empty {
  background: transparent;
  border-color: transparent;
}
.calendar-table td.is-today {
  background: var(--accent-gold-muted);
}
.calendar-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  min-height: 60px;
  padding: 4px 2px;
  cursor: default;
}
.calendar-cell.positive {
  background: rgba(231, 76, 76, 0.04);
}
.calendar-cell.negative {
  background: rgba(39, 174, 96, 0.04);
}
.cal-day-num {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-display);
  line-height: 1;
}
.cal-return {
  font-family: var(--font-display);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: -0.01em;
  line-height: 1;
  word-break: keep-all;
}
.cal-return.up {
  color: var(--up-red);
}
.cal-return.down {
  color: var(--down-green);
}

/* ─── Empty State ─── */

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: var(--text-muted);
}
.empty-icon {
  width: 40px;
  height: 40px;
  margin-bottom: 8px;
  opacity: 0.4;
}
.empty p {
  font-size: 13px;
}
.empty-hint {
  font-size: 12px;
  opacity: 0.7;
}
</style>
