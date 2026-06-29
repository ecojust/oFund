<template>
  <div class="simulation">
    <div class="header">
      <button class="back-btn" @click="goBack">← 返回</button>
      <div class="fund-title">
        <span class="fund-code">{{ fundCode }}</span>
        <span v-if="fundName" class="fund-name">{{ fundName }}</span>
      </div>
    </div>

    <!-- Phase 1: Setup -->
    <template v-if="phase === 'setup'">
      <div class="setup">
        <div v-if="loading" class="loading-overlay">
          <div class="loading-spinner"></div>
          <span class="loading-text">获取数据中...</span>
        </div>

        <div class="setup-row">
          <div class="period-group">
            <button
              v-for="opt in periodOptions"
              :key="opt.value"
              class="period-btn"
              :class="{ active: period === opt.value }"
              @click="period = opt.value"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>

        <div v-if="dailyChanges.length" class="range-section">
          <div class="range-header">
            <span class="range-label">选择模拟时间范围</span>
            <span class="range-info">{{ dailyChanges.length }} 个交易日</span>
          </div>
          <div class="date-pickers">
            <input
              type="date"
              v-model="rangeStartDate"
              :min="minDateStr"
              :max="maxDateStr"
              class="date-picker"
            />
            <span class="range-sep">至</span>
            <input
              type="date"
              v-model="rangeEndDate"
              :min="minDateStr"
              :max="maxDateStr"
              class="date-picker"
            />
          </div>
          <div v-if="rangeDayCount > 1" class="range-hint">
            共 <strong>{{ rangeDayCount }}</strong> 个交易日可供模拟
          </div>
          <button
            class="btn-primary"
            :disabled="rangeDayCount < 2"
            @click="startSimulation"
          >
            开始模拟
          </button>
        </div>
      </div>
    </template>

    <!-- Phase 2: Simulation -->
    <template v-if="phase === 'simulate'">
      <div class="sim-body">
        <div class="sim-left">
          <div class="day-header">
            <span class="day-progress"
              >第 {{ currentStep + 1 }} 天 / 共 {{ simDays.length }} 天</span
            >
            <span class="day-date">{{ currentDay.date }}</span>
          </div>

          <div
            class="change-card"
            :class="currentDay.change >= 0 ? 'up' : 'down'"
          >
            <div class="change-label">今日涨跌</div>
            <div class="change-value">
              {{ currentDay.change >= 0 ? "+" : ""
              }}{{ currentDay.change.toFixed(2) }}%
            </div>
          </div>

          <div class="invest-section">
            <div class="invest-label">今日投资额</div>
            <div class="invest-presets">
              <button
                v-for="amt in presets"
                :key="amt"
                class="preset-btn"
                :class="{ active: currentInvestment === amt }"
                @click="currentInvestment = amt"
              >
                {{
                  amt === 0 ? "不投" : `¥${amt.toLocaleString()}`
                }}
              </button>
            </div>
            <div class="invest-controls">
              <div class="number-input">
                <input
                  type="number"
                  v-model.number="currentInvestment"
                  min="0"
                  step="100"
                  class="number-field"
                />
                <div class="number-arrows">
                  <button
                    class="number-arrow up"
                    @click="currentInvestment += 100"
                    tabindex="-1"
                  >▲</button>
                  <button
                    class="number-arrow down"
                    @click="currentInvestment = Math.max(0, currentInvestment - 100)"
                    tabindex="-1"
                  >▼</button>
                </div>
              </div>
              <button class="btn-primary" @click="confirmDay">确认</button>
              <button class="btn-outline" :disabled="aiLoading" @click="askAI">
                <span v-if="aiLoading" class="spinner"></span>
                问问AI
              </button>
            </div>
          </div>

          <div class="invest-actions">
            <button
              class="btn-outline ai-auto-btn"
              :disabled="aiAutoRunning"
              @click="runAiAutoSimulation"
            >
              <span v-if="aiAutoRunning" class="spinner"></span>
              AI 自动模拟
            </button>
          </div>

          <div v-if="aiReason" class="ai-reply">
            <div class="ai-reply-reason">{{ aiReason }}</div>
            <div class="ai-reply-putin">
              建议投资 <strong>¥{{ formatMoney(aiPutin) }}</strong
              >，已填入输入框
            </div>
          </div>

          <div v-if="aiPrompt" class="ai-prompt" @click="togglePrompt">
            <span class="ai-prompt-toggle">{{
              promptExpanded ? "收起" : "查看发送给 AI 的 prompt"
            }}</span>
            <div v-show="promptExpanded" class="ai-prompt-text">
              {{ aiPrompt }}
            </div>
          </div>

          <div v-if="totalInvested > 0" class="position-bar">
            <span class="position-label">仓位</span>
            <div class="position-track">
              <div
                class="position-fill"
                :style="{ width: positionPct + '%' }"
              ></div>
            </div>
            <span class="position-text"
              >¥{{ formatMoney(totalInvested) }} / ¥{{
                formatMoney(totalBudget)
              }}</span
            >
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
              <span
                class="mono"
                :class="{ warn: totalBudget - totalInvested < 5000 }"
                >¥{{
                  formatMoney(Math.max(0, totalBudget - totalInvested))
                }}</span
              >
            </div>
            <div class="summary-row">
              <span>累计盈亏</span>
              <span class="mono" :class="cumulativePnl >= 0 ? 'up' : 'down'">
                {{ cumulativePnl >= 0 ? "+" : "" }}¥{{
                  formatMoney(cumulativePnl)
                }}
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

    <!-- Phase 3: Complete Dialog -->
    <div v-if="showCompleteDialog" class="dialog-overlay">
      <div class="dialog-panel">
        <div class="dialog-header">
          <h3>模拟结束</h3>
          <button class="dialog-close" @click="showCompleteDialog = false">×</button>
        </div>
        <div class="dialog-body">
          <div class="dialog-stats">
            <div class="dialog-stat">
              <span class="dialog-stat-label">模拟天数</span>
              <span class="dialog-stat-value">{{ simDays.length }} 天</span>
            </div>
            <div class="dialog-stat">
              <span class="dialog-stat-label">总投资额</span>
              <span class="dialog-stat-value mono"
                >¥{{ formatMoney(totalBudget) }}</span
              >
            </div>
            <div class="dialog-stat">
              <span class="dialog-stat-label">累计投入</span>
              <span class="dialog-stat-value mono"
                >¥{{ formatMoney(totalInvested) }}</span
              >
            </div>
            <div class="dialog-stat">
              <span class="dialog-stat-label">累计盈亏</span>
              <span
                class="dialog-stat-value mono"
                :class="cumulativePnl >= 0 ? 'up' : 'down'"
              >
                {{ cumulativePnl >= 0 ? "+" : "" }}¥{{ formatMoney(cumulativePnl) }}
              </span>
            </div>
            <div class="dialog-stat">
              <span class="dialog-stat-label">最终价值</span>
              <span class="dialog-stat-value mono"
                >¥{{ formatMoney(currentValue) }}</span
              >
            </div>
            <div v-if="totalInvested > 0" class="dialog-stat">
              <span class="dialog-stat-label">收益率</span>
              <span
                class="dialog-stat-value"
                :class="cumulativePnl >= 0 ? 'up' : 'down'"
              >
                {{ ((cumulativePnl / totalInvested) * 100).toFixed(2) }}%
              </span>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn-primary" @click="resetAll">重新开始</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  watch,
  onBeforeUnmount,
  nextTick,
} from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import * as echarts from "echarts";
import OpencodeService from "../service/opencode";

interface HistoryPoint {
  timestamp: number;
  value: number;
}

interface FundHistory {
  fund_code: string;
  fund_name: string;
  data: HistoryPoint[];
}

type Phase = "setup" | "simulate" | "complete";

const route = useRoute();
const router = useRouter();

const fundCode = ref("");
const fundName = ref("");
const period = ref("1m");
const loading = ref(false);
const historyData = ref<HistoryPoint[]>([]);
const phase = ref<Phase>("setup");
const showCompleteDialog = ref(false);

const periodOptions = [
  { value: "1m", label: "1个月" },
  { value: "3m", label: "3个月" },
  { value: "6m", label: "6个月" },
  { value: "1y", label: "1年" },
  { value: "all", label: "全部" },
];

function goBack() {
  router.push("/");
}

function formatDate(ts: number) {
  const d = new Date(ts);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function parseDate(str: string): Date {
  const [y, m, d] = str.split("-").map(Number);
  return new Date(y, m - 1, d);
}

// ─── Daily Changes ───

const dailyChanges = computed(() => {
  const sorted = [...historyData.value].sort(
    (a, b) => a.timestamp - b.timestamp,
  );
  const changes: { date: string; change: number }[] = [];
  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1].value;
    const cur = sorted[i].value;
    const change = ((cur - prev) / (100 + prev)) * 100;
    changes.push({ date: formatDate(sorted[i].timestamp), change });
  }
  return changes;
});

const minDate = computed(() => {
  if (!dailyChanges.value.length) return new Date();
  return parseDate(dailyChanges.value[0].date);
});

const maxDate = computed(() => {
  if (!dailyChanges.value.length) return new Date();
  return parseDate(dailyChanges.value[dailyChanges.value.length - 1].date);
});

const minDateStr = computed(() => formatDate(minDate.value.getTime()));
const maxDateStr = computed(() => formatDate(maxDate.value.getTime()));

// ─── Range Selection ───

const rangeStartDate = ref<string>("");
const rangeEndDate = ref<string>("");

function findLastIndex<T>(arr: T[], fn: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (fn(arr[i])) return i;
  }
  return -1;
}

const rangeDayCount = computed(() => {
  if (!rangeStartDate.value || !rangeEndDate.value) return 0;
  const startIdx = dailyChanges.value.findIndex(
    (d) => d.date >= rangeStartDate.value,
  );
  const endIdx = findLastIndex(
    dailyChanges.value,
    (d) => d.date <= rangeEndDate.value,
  );
  if (startIdx === -1 || endIdx === -1 || endIdx <= startIdx) return 0;
  return endIdx - startIdx + 1;
});

// ─── Simulation State ───

interface SimulationDay {
  date: string;
  change: number;
  investment: number;
  pnl: number;
  position: number;
  aiReason: string;
}

const simDays = ref<SimulationDay[]>([]);
const currentStep = ref(0);
const currentInvestment = ref(1000);
const presets = [0, 100, 500, 1000, 5000, 10000];
const currentDay = computed(
  () =>
    simDays.value[currentStep.value] ?? {
      date: "",
      change: 0,
      investment: 0,
      pnl: 0,
      position: 0,
    },
);

const totalInvested = computed(() =>
  simDays.value.reduce((sum, d) => sum + d.investment, 0),
);

const cumulativePnl = computed(() =>
  simDays.value.reduce((sum, d) => sum + d.pnl, 0),
);

const totalBudget = 50000;
const positionPct = computed(() =>
  Math.min((totalInvested.value / totalBudget) * 100, 100),
);
const currentValue = computed(() => totalInvested.value + cumulativePnl.value);

function formatMoney(n: number): string {
  return n.toLocaleString("zh-CN", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

// ─── Chart ───

const chartRef = ref<HTMLElement | null>(null);
let chartInstance: echarts.ECharts | null = null;

const chartPoints = computed(() => {
  if (!simDays.value.length || !historyData.value.length) return [];
  const sorted = [...historyData.value].sort(
    (a, b) => a.timestamp - b.timestamp,
  );
  const firstSimDate = simDays.value[0].date;
  const firstPointIdx = sorted.findIndex(
    (p) => formatDate(p.timestamp) === firstSimDate,
  );
  if (firstPointIdx < 0) return sorted.slice(0, 2);
  const showUpTo = Math.min(
    firstPointIdx + currentStep.value,
    sorted.length - 1,
  );
  const result = sorted.slice(0, 1);
  for (let i = firstPointIdx; i <= showUpTo; i++) {
    result.push(sorted[i]);
  }
  return result;
});

function buildChartOption() {
  const points = chartPoints.value;
  if (!points.length) return {};

  const currentTs =
    currentStep.value < simDays.value.length
      ? parseDate(simDays.value[currentStep.value].date).getTime()
      : points[points.length - 1].timestamp;

  const markLines: any[] = [
    {
      xAxis: currentTs,
      label: { show: false },
      lineStyle: { color: "rgba(255,255,255,0.25)", type: "dashed", width: 1 },
      symbol: "none",
    },
  ];

  const markPoints: any[] = [];
  simDays.value.forEach((day) => {
    const pt = points.find((p) => formatDate(p.timestamp) === day.date);
    if (!pt) return;
    const isZero = day.investment === 0;
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
        formatter: isZero
          ? "0"
          : `¥${day.investment >= 10000 ? (day.investment / 10000).toFixed(1) + "w" : day.investment.toFixed(0)}`,
        position: "top",
        color: "#D4A84B",
        fontSize: 10,
        fontFamily: "SF Mono, monospace",
      },
    });
  });

  return {
    backgroundColor: "transparent",
    grid: { left: 50, right: 16, top: 28, bottom: 28 },
    tooltip: {
      trigger: "axis",
      formatter: (params: any) => {
        const p = params[0];
        if (!p) return "";
        return `${formatDate(p.data[0])}<br/>累计收益率: ${p.data[1].toFixed(2)}%`;
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
    series: [
      {
        type: "line",
        data: points.map((p) => [p.timestamp, p.value]),
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
      },
    ],
  };
}

function renderChart() {
  if (!chartRef.value || !chartPoints.value.length) return;
  if (!chartInstance) {
    chartInstance = echarts.init(chartRef.value, undefined, {
      renderer: "canvas",
    });
  }
  chartInstance.setOption(buildChartOption(), true);
  chartInstance.resize();
}

function updateChartMarker() {
  if (!chartInstance || !chartPoints.value.length) return;
  chartInstance.setOption(buildChartOption(), true);
}

// ─── Watch chart ───

watch(currentStep, () => {
  nextTick(() => updateChartMarker());
});

watch(
  simDays,
  () => {
    nextTick(() => updateChartMarker());
  },
  { deep: true },
);

watch(period, () => {
  if (fundCode.value && phase.value === "setup") {
    fetchData();
  }
});

// ─── Actions ───

async function fetchData() {
  loading.value = true;
  try {
    const result = await invoke<FundHistory>("get_fund_history", {
      fundCode: fundCode.value,
      period: period.value,
    });
    fundName.value = result.fund_name;
    historyData.value = result.data;
    if (dailyChanges.value.length) {
      rangeStartDate.value = dailyChanges.value[0].date;
      rangeEndDate.value =
        dailyChanges.value[dailyChanges.value.length - 1].date;
    }
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

function startSimulation() {
  const startIdx = dailyChanges.value.findIndex(
    (d) => d.date >= rangeStartDate.value,
  );
  const endIdx = findLastIndex(
    dailyChanges.value,
    (d) => d.date <= rangeEndDate.value,
  );
  if (startIdx === -1 || endIdx === -1 || endIdx <= startIdx) return;

  simDays.value = dailyChanges.value.slice(startIdx, endIdx + 1).map((d) => ({
    date: d.date,
    change: d.change,
    investment: 0,
    pnl: 0,
    position: 0,
    aiReason: "",
  }));
  currentStep.value = 0;
  currentInvestment.value = 1000;
  phase.value = "simulate";
  nextTick(() => renderChart());
}

function confirmDay() {
  const step = currentStep.value;
  const remaining = totalBudget - totalInvested.value;
  let inv = currentInvestment.value || 0;
  if (inv > remaining) inv = remaining;
  currentInvestment.value = inv;

  const day = simDays.value[step];
  day.investment = inv;

  if (step === 0) {
    day.pnl = 0;
    day.position = inv;
  } else {
    const prevPosition = simDays.value[step - 1].position;
    day.pnl = prevPosition * (day.change / 100);
    day.position = prevPosition + day.pnl + inv;
  }

  if (step >= simDays.value.length - 1) {
    showCompleteDialog.value = true;
  } else {
    currentStep.value++;
  }
}

function resetAll() {
  phase.value = "setup";
  showCompleteDialog.value = false;
  simDays.value = [];
  currentStep.value = 0;
  chartInstance?.dispose();
  chartInstance = null;
}

// ─── AI Assistant ───

const aiLoading = ref(false);
const aiReason = ref("");
const aiPutin = ref(0);
const aiPrompt = ref("");
const promptExpanded = ref(false);
let aiInitialized = false;

async function askAI() {
  aiLoading.value = true;
  try {
    if (!aiInitialized) {
      await OpencodeService.initialize(fundCode.value);
      aiInitialized = true;
    }

    const remaining = totalBudget - totalInvested.value;
    const pastDays = simDays.value.slice(0, currentStep.value + 1);
    const historyText = pastDays
      .map(
        (d, i) =>
          `第${i + 1}天 ${d.date}: 涨跌 ${d.change >= 0 ? "+" : ""}${d.change.toFixed(2)}%, 投入 ¥${d.investment}, 盈亏 ¥${d.pnl.toFixed(2)}, 持仓 ¥${d.position.toFixed(2)}`,
      )
      .join("\n");

    const prompt = `你是量化基金投资顾问。请参考以下经典量化策略，结合数据分析给出今日建议投资额度。

## 经典量化策略参考

1. **网格交易法** — 设定基准价，每跌一定幅度加仓，每涨一定幅度减仓，适合震荡市
2. **定投策略 (DCA)** — 无视波动定期定额投入，平滑成本
3. **动量策略** — 趋势向上时加仓，趋势向下时减仓或空仓
4. **均值回归** — 连续上涨后降低仓位（恐高），连续下跌后增加仓位（贪婪）
5. **金字塔建仓** — 初始轻仓，越跌越加大仓位，越涨越减小仓位
6. **凯利公式** — 根据近期胜率与盈亏比计算最优仓位比例
7. **股债平衡** — 将总投资预算视为一个组合，维持股债目标比例，偏离时再平衡
8. **趋势跟踪 (移动均线)** — 价格在均线上方做多，下方减仓

## 当前持仓与市场数据

基金代码：${fundCode.value}
总投资预算：¥${formatMoney(totalBudget)}
今日涨跌幅：${currentDay.value.change >= 0 ? "+" : ""}${currentDay.value.change.toFixed(2)}%
已投入金额：¥${formatMoney(totalInvested.value)}
累计盈亏：${cumulativePnl.value >= 0 ? "+" : ""}¥${formatMoney(cumulativePnl.value)}
当前总价值：¥${formatMoney(currentValue.value)}
剩余可用资金：¥${formatMoney(remaining)}
仓位比例：${positionPct.value.toFixed(1)}%

## 每日明细
${historyText}

请综合以上策略，基于今日涨跌幅和当前持仓给出建议。返回可以直接解析的JSON格式数据（不要其他文字）：
{"reason":"说明你参考了哪个策略、分析逻辑和理由","putin":建议金额}`;

    aiPrompt.value = prompt;
    const reply = await OpencodeService.sendMessage(prompt);
    try {
      const parsed = JSON.parse(reply);
      aiReason.value = parsed.reason || "";
      const recommended = Math.min(
        Math.max(Number(parsed.putin) || 0, 0),
        remaining,
      );
      aiPutin.value = recommended;
      currentInvestment.value = recommended;
      simDays.value[currentStep.value].aiReason = aiReason.value;
    } catch {
      aiReason.value = reply;
      const match = reply.match(/\d+/);
      if (match) {
        const recommended = Math.min(Math.max(Number(match[0]), 0), remaining);
        aiPutin.value = recommended;
        currentInvestment.value = recommended;
      }
      simDays.value[currentStep.value].aiReason = aiReason.value;
    }
  } catch (e) {
    console.error("AI 建议失败", e);
  } finally {
    aiLoading.value = false;
  }
}

// ─── AI Auto Simulation ───

const aiAutoRunning = ref(false);

async function runAiAutoSimulation() {
  aiAutoRunning.value = true;
  currentStep.value = 0;
  currentInvestment.value = 0;
  for (const d of simDays.value) {
    d.investment = 0;
    d.pnl = 0;
    d.position = 0;
    d.aiReason = "";
  }
  chartInstance?.dispose();
  chartInstance = null;
  nextTick(() => renderChart());

  await OpencodeService.killAll();
  aiInitialized = false;

  try {
    while (currentStep.value < simDays.value.length) {
      await askAI();
      confirmDay();
      if (showCompleteDialog.value) break;
      await new Promise((r) => setTimeout(r, 100));
    }
  } catch (e) {
    console.error("AI 自动模拟失败", e);
  } finally {
    aiAutoRunning.value = false;
  }
}

function togglePrompt() {
  promptExpanded.value = !promptExpanded.value;
}

// ─── Lifecycle ───

onMounted(() => {
  fundCode.value = route.params.code as string;
  window.addEventListener("resize", handleResize);
  if (fundCode.value) {
    fetchData();
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", handleResize);
  chartInstance?.dispose();
  chartInstance = null;
});

function handleResize() {
  chartInstance?.resize();
}

watch(
  () => route.params.code,
  (code) => {
    if (code && code !== fundCode.value) {
      fundCode.value = code as string;
      fundName.value = "";
      resetAll();
    }
  },
);
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

/* ─── Shared button styles ─── */

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

.btn-outline {
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
  background: transparent;
  color: var(--accent-gold);
  line-height: 1;
}
.btn-outline:hover:not(:disabled) {
  background: var(--accent-gold-muted);
}
.btn-outline:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

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

/* ─── Period Group ─── */

.period-group {
  display: flex;
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

/* ─── Setup ─── */

.setup {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  padding: 40px 16px;
  position: relative;
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
  height: 28px;
  padding: 0 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-display);
  font-size: 12px;
  outline: none;
  transition: border-color 0.2s ease;
  color-scheme: dark;
}
.date-picker:focus {
  border-color: var(--accent-gold);
}
.date-picker::-webkit-calendar-picker-indicator {
  filter: invert(0.6);
  cursor: pointer;
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

/* ─── Loading Overlay ─── */

.loading-overlay {
  position: absolute;
  inset: 0;
  background: rgba(11, 11, 15, 0.7);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  z-index: 10;
  backdrop-filter: blur(2px);
  border-radius: var(--radius-lg);
}
.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--accent-gold-muted);
  border-top-color: var(--accent-gold);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.loading-text {
  font-size: 13px;
  color: var(--text-secondary);
  font-family: var(--font-body);
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
.preset-btn {
  font-family: var(--font-display);
  font-size: 12px;
  padding: 0 10px;
  height: 28px;
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
  background: transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s ease;
}
.preset-btn:hover {
  border-color: var(--accent-gold);
  color: var(--accent-gold);
}
.preset-btn.active {
  border-color: var(--accent-gold);
  background: var(--accent-gold-muted);
  color: var(--accent-gold);
}
.invest-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Number Input */
.number-input {
  display: flex;
  height: 28px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--bg-surface);
  flex: 1;
  transition: border-color 0.2s ease;
}
.number-input:focus-within {
  border-color: var(--accent-gold);
}
.number-field {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-family: var(--font-display);
  font-size: 12px;
  padding: 0 8px;
  text-align: right;
  outline: none;
  -moz-appearance: textfield;
}
.number-field::-webkit-inner-spin-button,
.number-field::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.number-arrows {
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-default);
  flex-shrink: 0;
}
.number-arrow {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 7px;
  padding: 0 7px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transition: all 0.15s ease;
}
.number-arrow:hover {
  color: var(--accent-gold);
  background: var(--accent-gold-muted);
}
.number-arrow.up {
  border-bottom: 1px solid var(--border-default);
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
  background: rgba(255, 255, 255, 0.06);
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

/* ─── Invest Actions ─── */

.invest-actions {
  display: flex;
  gap: 8px;
}
.ai-auto-btn {
  flex: 1;
}

/* ─── AI Reply ─── */

.ai-reply {
  padding: 10px 12px;
  background: rgba(212, 168, 75, 0.06);
  border: 1px solid rgba(212, 168, 75, 0.2);
  border-radius: var(--radius-sm);
}
.ai-reply-reason {
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary);
}
.ai-reply-putin {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 6px;
}
.ai-reply-putin strong {
  color: var(--accent-gold);
  font-family: var(--font-display);
}

/* ─── AI Prompt ─── */

.ai-prompt {
  margin-top: 8px;
  cursor: pointer;
}
.ai-prompt-toggle {
  font-size: 11px;
  color: var(--text-muted);
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 2px;
}
.ai-prompt-text {
  margin-top: 6px;
  padding: 8px 10px;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-secondary);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: var(--font-display);
  max-height: 200px;
  overflow-y: auto;
}

/* Navigation */
.nav-buttons {
  display: flex;
  justify-content: space-between;
  padding-top: 4px;
}

/* ─── Dialog ─── */

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.15s ease;
}
.dialog-panel {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  animation: scaleIn 0.15s ease;
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-subtle);
}
.dialog-header h3 {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}
.dialog-close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 20px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
  transition: color 0.15s ease;
}
.dialog-close:hover {
  color: var(--text-primary);
}
.dialog-body {
  padding: 20px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border-subtle);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
@keyframes scaleIn {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

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
</style>
