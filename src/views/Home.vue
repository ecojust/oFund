<template>
  <div class="home">
    <div class="brand">
      <h1><span class="brand-o">o</span>Fund</h1>
      <span class="tagline">基金策略分析工具</span>
      <div class="brand-spacer"></div>
      <button class="account-btn" @click="openAccountDialog" title="账户配置">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="8" r="4" />
          <path d="M4 20c0-4 3.5-7 8-7s8 3 8 7" />
        </svg>
      </button>
    </div>

    <div class="toolbar">
      <div class="toolbar-left">
        <div class="search-wrap">
          <svg
            class="search-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.35-4.35" />
          </svg>
          <input
            v-model="search"
            placeholder="搜索基金代码或名称..."
            class="search-input"
          />
        </div>
        <span v-if="funds.length" class="total-count">
          共 {{ funds.length.toLocaleString() }} 只基金
        </span>
        <span
          v-if="search && filteredFunds.length !== funds.length"
          class="filtered-count"
        >
          / {{ filteredFunds.length.toLocaleString() }} 只
        </span>

        <div class="filter-bar" v-show="funds.length">
          <div class="view-tabs">
            <button
              class="view-tab"
              :class="{ active: !showFavoritesOnly }"
              @click="showFavoritesOnly = false"
            >
              全部
            </button>
            <button
              class="view-tab"
              :class="{ active: showFavoritesOnly }"
              @click="showFavoritesOnly = true"
            >
              收藏
            </button>
          </div>
          <span v-if="showFavoritesOnly" class="filter-count"
            >{{ filteredFunds.length.toLocaleString() }} 只</span
          >
        </div>
      </div>
      <div class="toolbar-right" v-show="!showFavoritesOnly">
        <button class="btn btn-ghost" @click="openHistoryDir">历史缓存</button>

        <div class="btn-group">
          <button
            class="btn btn-outline"
            :disabled="historyLoading || !funds.length"
            @click="fetchAllHistory"
          >
            <span v-if="historyLoading" class="spinner"></span>
            {{
              historyLoading ? historyProgressText : `批量获取(${periodLabel})`
            }}
          </button>
          <button
            class="btn btn-outline btn-arrow"
            :disabled="historyLoading || !funds.length"
            @click="togglePeriodMenu"
            ref="periodToggleRef"
          >
            <svg viewBox="0 0 24 24" width="10" height="10" fill="currentColor">
              <path d="M7 10l5 5 5-5z" />
            </svg>
          </button>
          <div v-if="showPeriodMenu" class="dropdown-menu">
            <button
              v-for="opt in periodOptions"
              :key="opt.value"
              class="dropdown-item"
              :class="{ active: batchPeriod === opt.value }"
              @click="selectPeriod(opt.value)"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>

        <button
          class="btn btn-primary"
          :disabled="loading"
          @click="fetchAllFunds"
        >
          <span v-if="loading" class="spinner"></span>
          {{ loading ? progressText : "更新基金列表" }}
        </button>
      </div>
    </div>

    <div class="table-container" ref="tableContainerRef">
      <div class="table-header">
        <div class="tr">
          <div class="th col-code">基金代码</div>
          <div class="th col-name">基金名称</div>
          <div class="th col-company">基金公司</div>
          <div class="th col-action">操作</div>
          <div class="th col-status">已获取</div>
        </div>
      </div>

      <div class="table-body" ref="tableBodyRef" @scroll="onTableScroll">
        <template v-if="filteredFunds.length">
          <div class="virtual-sizer" :style="{ height: totalHeight + 'px' }">
            <div
              class="virtual-content"
              :style="{ transform: `translateY(${offsetY}px)` }"
            >
              <div
                v-for="row in visibleRows"
                :key="row.id"
                class="tr"
                :style="{ height: ROW_HEIGHT + 'px' }"
              >
                <div class="td col-code">
                  <span class="code-text">{{ row.id }}</span>
                </div>
                <div class="td col-name" :title="row.name">
                  <span class="name-text">{{ row.name }}</span>
                </div>
                <div class="td col-company">
                  <span class="company-text">{{ row.company_name }}</span>
                </div>
                <div class="td col-action">
                  <button
                    class="star-btn"
                    :class="{ starred: isFavorite(row.id) }"
                    @click="toggleFavorite(row.id)"
                    :title="isFavorite(row.id) ? '取消收藏' : '收藏'"
                  >
                    <svg
                      viewBox="0 0 24 24"
                      width="14"
                      height="14"
                      fill="currentColor"
                    >
                      <path
                        d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
                      />
                    </svg>
                  </button>
                  <button class="action-btn" @click="viewHistory(row.id)">
                    走势
                  </button>
                  <span class="action-divider"></span>
                  <button class="action-btn" @click="viewSimulation(row.id)">
                    量化模拟
                  </button>
                  <button
                    v-if="investedCodes.has(row.id)"
                    class="coin-btn"
                    @click="showInvestDetail(row.id)"
                    title="投资明细"
                  >
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                      <circle cx="12" cy="12" r="10" />
                      <text x="12" y="16" text-anchor="middle" font-size="14" font-weight="bold" fill="#0b0b0f">¥</text>
                    </svg>
                  </button>
                </div>
                <div class="td col-status">
                  <span class="status-cell">
                    <span
                      class="status-dot"
                      :class="hasHistory(row.id) ? 'yes' : 'no'"
                    ></span>
                    {{ hasHistory(row.id) ? "有" : "无" }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </template>
        <div v-else class="empty-state">
          <svg
            class="empty-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <rect x="3" y="3" width="18" height="18" rx="2" />
            <path d="M9 3v18M15 3v18M3 9h18M3 15h18" />
          </svg>
          <p v-if="search">没有匹配的基金</p>
          <p v-else-if="showFavoritesOnly">还没有收藏的基金</p>
          <p v-else>暂无数据</p>
          <p v-if="!search && !showFavoritesOnly" class="empty-hint">
            点击「更新基金列表」获取数据
          </p>
          <p v-if="!search && showFavoritesOnly" class="empty-hint">
            点击基金行中的星标按钮收藏
          </p>
        </div>
      </div>

      <div v-if="loading" class="loading-overlay">
        <div class="loading-spinner"></div>
      </div>
    </div>

    <!-- Account Dialog: investment list -->
    <div v-if="showAccountDialog" class="dialog-overlay" @click.self="showAccountDialog = false">
      <div class="dialog-panel account-dialog">
        <div class="dialog-header">
          <h3>基金投资明细</h3>
          <button class="dialog-close" @click="showAccountDialog = false">×</button>
        </div>
        <div class="dialog-body">
          <div v-if="investments.length === 0" class="empty-investments">
            <p>暂无投资记录</p>
          </div>
          <div v-else class="investment-list">
            <div
              v-for="(inv, idx) in investments"
              :key="inv.code"
              class="investment-row"
            >
              <div class="inv-info">
                <span class="inv-code">{{ inv.code }}</span>
                <span class="inv-name">{{ inv.name }}</span>
                <span class="inv-total">投资计划 ¥{{ getFundTotal(inv).toLocaleString() }}</span>
              </div>
              <button class="inv-config-btn" @click="openSchedule(idx)">配置</button>
              <button class="inv-remove" @click="removeInvestment(idx)" title="移除">×</button>
            </div>
          </div>
          <button class="btn btn-primary add-investment-btn" :disabled="!favoriteFunds.length" @click="openAddSchedule">
            添加新基金...
          </button>
        </div>
        <div class="dialog-footer">
          <span class="total-investment">总计：¥{{ grandTotal.toLocaleString() }}</span>
          <button class="btn-done" @click="showAccountDialog = false">完成</button>
        </div>
      </div>
    </div>

    <!-- Schedule Dialog: calendar with daily amounts -->
    <div v-if="showScheduleDialog" class="dialog-overlay" @click.self="showScheduleDialog = false">
      <div class="dialog-panel schedule-dialog">
        <div class="dialog-header">
          <h3>{{ editingFund ? editingFund.name : '配置投资计划' }}</h3>
          <button class="dialog-close" @click="closeSchedule">×</button>
        </div>
        <div class="dialog-body">
          <div v-if="!editingFund" class="select-fund-area">
            <select v-model="scheduleFundCode" class="fund-select">
              <option value="" disabled>选择已收藏的基金...</option>
              <option
                v-for="f in favoriteFunds"
                :key="f.id"
                :value="f.id"
              >
                {{ f.id }} - {{ f.name }}
              </option>
            </select>
            <button class="btn btn-primary" :disabled="!scheduleFundCode" @click="loadFundForSchedule">
              下一步
            </button>
          </div>
          <template v-else>
            <div class="schedule-fund-bar">
              <span class="schedule-fund-code">{{ editingFund.code }}</span>
              <button class="btn btn-ghost" @click="fetchHistoryForSchedule">刷新数据</button>
            </div>
            <div v-if="scheduleHistoryLoading" class="loading-overlay">
              <div class="loading-spinner"></div>
            </div>
            <template v-if="scheduleHistory.length">
              <div class="calendar-toolbar">
                <button class="cal-nav" @click="schedulePrevMonth">‹</button>
                <h3>{{ scheduleCalendarDate.getFullYear() }}年{{ scheduleCalendarDate.getMonth() + 1 }}月</h3>
                <button class="cal-nav" @click="scheduleNextMonth">›</button>
              </div>
              <table class="calendar-table">
                <thead>
                  <tr>
                    <th v-for="d in weekDays" :key="d">{{ d }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(week, wi) in scheduleCalendarDays" :key="wi">
                    <td
                      v-for="(cell, ci) in week"
                      :key="ci"
                      :class="{ 'is-empty': cell.isEmpty }"
                    >
                      <div
                        v-if="!cell.isEmpty"
                        class="cal-cell"
                        :class="getScheduleCellClass(cell.dateStr)"
                        @click="openDayInput(cell.dateStr)"
                      >
                        <span class="cal-day-num">{{ cell.day }}</span>
                        <span v-if="getDailyReturn(cell.dateStr) !== null" class="cal-return" :class="getReturnClass(getDailyReturn(cell.dateStr)!)">
                          {{ getDailyReturn(cell.dateStr)!.toFixed(2) }}%
                        </span>
                        <span v-if="getDayAmount(cell.dateStr)" class="cal-amount">
                          ¥{{ getDayAmount(cell.dateStr) }}
                        </span>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </template>
            <div v-if="!scheduleHistory.length && !scheduleHistoryLoading" class="empty-state">
              <p>暂无数据，请先获取历史数据</p>
            </div>
          </template>
        </div>
        <div class="dialog-footer">
          <span class="total-investment" v-if="editingFund">
            已配置：¥{{ getFundTotal(editingFund).toLocaleString() }}
          </span>
          <button class="btn-done" @click="closeSchedule">完成</button>
        </div>
      </div>
    </div>

    <!-- Day amount input popover -->
    <div
      v-if="showDayInput && editingFund"
      class="day-input-overlay"
      @click.self="showDayInput = false"
    >
      <div class="day-input-panel">
        <div class="day-input-header">
          <span class="day-input-date">{{ dayInputDate }}</span>
          <span v-if="getDailyReturn(dayInputDate) !== null" class="day-input-return" :class="getReturnClass(getDailyReturn(dayInputDate)!)">
            收益率: {{ getDailyReturn(dayInputDate)!.toFixed(2) }}%
          </span>
        </div>
        <div class="day-input-body">
          <label>投资金额（¥）</label>
          <input
            ref="dayInputRef"
            type="number"
            v-model="dayInputAmount"
            class="day-input-field"
            min="0"
            step="100"
            placeholder="输入金额"
            @keydown.enter="confirmDayInput"
            @keydown.escape="showDayInput = false"
          />
        </div>
        <div class="day-input-footer">
          <button class="btn" @click="showDayInput = false">取消</button>
          <button class="btn-primary" @click="confirmDayInput">确定</button>
        </div>
      </div>
    </div>

    <!-- Invest Detail Dialog -->
    <div v-if="showInvestDetailDialog && investDetailFund" class="dialog-overlay" @click.self="showInvestDetailDialog = false">
      <div class="dialog-panel invest-detail-dialog">
        <div class="dialog-header">
          <h3>{{ investDetailFund.code }} - {{ investDetailFund.name }}</h3>
          <button class="dialog-close" @click="showInvestDetailDialog = false">×</button>
        </div>
        <div class="dialog-body">
          <div v-if="detailHistoryLoading" class="loading-overlay">
            <div class="loading-spinner"></div>
          </div>
          <template v-if="detailHistory.length">
            <div class="calendar-toolbar">
              <button class="cal-nav" @click="detailPrevMonth">‹</button>
              <h3>{{ detailCalendarDate.getFullYear() }}年{{ detailCalendarDate.getMonth() + 1 }}月</h3>
              <button class="cal-nav" @click="detailNextMonth">›</button>
            </div>
            <table class="calendar-table">
              <thead>
                <tr>
                  <th v-for="d in weekDays" :key="d">{{ d }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(week, wi) in detailCalendarDays" :key="wi">
                  <td v-for="(cell, ci) in week" :key="ci" :class="{ 'is-empty': cell.isEmpty }">
                    <div v-if="!cell.isEmpty" class="cal-cell" :class="getDetailCellClass(cell.dateStr)">
                      <span class="cal-day-num">{{ cell.day }}</span>
                      <span v-if="getDetailReturn(cell.dateStr) !== null" class="cal-return" :class="getReturnClass(getDetailReturn(cell.dateStr)!)">
                        {{ getDetailReturn(cell.dateStr)!.toFixed(2) }}%
                      </span>
                      <span v-if="getDetailAmount(cell.dateStr)" class="cal-amount">
                        ¥{{ getDetailAmount(cell.dateStr) }}
                      </span>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
            <div class="invest-detail-total">
              合计：¥{{ investDetailTotal.toLocaleString() }}
            </div>
          </template>
          <div v-else-if="!detailHistoryLoading" class="empty-state">
            <p>暂无历史数据</p>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn-primary" @click="showInvestDetailDialog = false">关闭</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface FundItem {
  id: string;
  name: string;
  company_id: string;
  company_name: string;
}

interface CrawlProgress {
  current: number;
  total: number;
  company_name: string;
  status: string;
}

interface HistoryCrawlProgress {
  current: number;
  total: number;
  fund_code: string;
  fund_name: string;
  status: string;
}

const ROW_HEIGHT = 44;
const OVERSCAN = 5;

let cache = undefined as FundItem[] | undefined;

const router = useRouter();
const loading = ref(false);
const progressText = ref("");
const historyLoading = ref(false);
const historyProgressText = ref("");
const batchPeriod = ref("1m");
const periodLabel = computed(() => {
  const map: Record<string, string> = {
    "1m": "1个月",
    "3m": "3个月",
    "6m": "6个月",
    "1y": "1年",
    all: "全部",
  };
  return map[batchPeriod.value] || "全部";
});
const periodOptions = [
  { value: "1m", label: "1个月" },
  { value: "3m", label: "3个月" },
  { value: "6m", label: "6个月" },
  { value: "1y", label: "1年" },
  { value: "all", label: "全部" },
];
const showPeriodMenu = ref(false);
const showFavoritesOnly = ref(false);
const search = ref("");

const funds = ref<FundItem[]>([]);
const cachedHistoryCodes = ref(new Set<string>());
const favorites = ref(new Set<string>());

const filteredFunds = computed(() => {
  let list = funds.value;
  if (showFavoritesOnly.value) {
    list = list.filter((f) => favorites.value.has(f.id));
  }
  const q = search.value.trim().toLowerCase();
  if (!q) return list;
  return list.filter(
    (f) => f.id.toLowerCase().includes(q) || f.name.toLowerCase().includes(q),
  );
});

interface FundInvestment {
  code: string;
  name: string;
  schedule: Record<string, number>;
}

const showAccountDialog = ref(false);

const investments = ref<FundInvestment[]>([]);

const favoriteFunds = computed(() => {
  return funds.value.filter((f) => favorites.value.has(f.id));
});

const investedCodes = computed(() => new Set(investments.value.map((i) => i.code)));

const grandTotal = computed(() =>
  investments.value.reduce((sum, i) => sum + getFundTotal(i), 0),
);

function getFundTotal(inv: FundInvestment) {
  return inv.schedule ? Object.values(inv.schedule).reduce((s, v) => s + v, 0) : 0;
}

async function loadAccount() {
  try {
    const data = await invoke<FundInvestment[]>("get_account");
    investments.value = data.map((item: any) => ({
      code: item.code,
      name: item.name,
      schedule: item.schedule || {},
    }));
  } catch (_e) {}
}

async function saveInvestments() {
  try {
    await invoke("save_account", { data: investments.value });
  } catch (_e) {}
}

function openAccountDialog() {
  showAccountDialog.value = true;
}

async function removeInvestment(idx: number) {
  investments.value.splice(idx, 1);
  await saveInvestments();
}

// ─── Schedule Dialog ───

interface HistoryPoint {
  timestamp: number;
  value: number;
}
interface FundHistory {
  fund_code: string;
  fund_name: string;
  data: HistoryPoint[];
}

const showScheduleDialog = ref(false);
const scheduleFundCode = ref("");
const editingIdx = ref(-1);
const editingFund = ref<FundInvestment | null>(null);
const scheduleHistory = ref<HistoryPoint[]>([]);
const scheduleHistoryLoading = ref(false);
const scheduleCalendarDate = ref(new Date());
const weekDays = ["日", "一", "二", "三", "四", "五", "六"];

const dailyReturnMap = computed(() => {
  const sorted = [...scheduleHistory.value].sort((a, b) => a.timestamp - b.timestamp);
  const map = new Map<string, number>();
  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1].value;
    const cur = sorted[i].value;
    const dailyReturn = ((cur - prev) / (100 + prev)) * 100;
    map.set(formatDate(sorted[i].timestamp), dailyReturn);
  }
  return map;
});

const dateTimestampMap = computed(() => {
  const map = new Map<string, number>();
  for (const point of scheduleHistory.value) {
    map.set(formatDate(point.timestamp), point.timestamp);
  }
  return map;
});

function getDailyReturn(dateStr: string): number | null {
  return dailyReturnMap.value.get(dateStr) ?? null;
}
function getReturnClass(ret: number) {
  return ret >= 0 ? "up" : "down";
}

const scheduleCalendarDays = computed(() => {
  const year = scheduleCalendarDate.value.getFullYear();
  const month = scheduleCalendarDate.value.getMonth();
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

function schedulePrevMonth() {
  const d = new Date(scheduleCalendarDate.value);
  d.setMonth(d.getMonth() - 1);
  scheduleCalendarDate.value = d;
}
function scheduleNextMonth() {
  const d = new Date(scheduleCalendarDate.value);
  d.setMonth(d.getMonth() + 1);
  scheduleCalendarDate.value = d;
}

function formatDate(ts: number) {
  const d = new Date(ts);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function getScheduleCellClass(dateStr: string) {
  const ret = getDailyReturn(dateStr);
  if (ret === null) return "";
  const hasAmount = getDayAmount(dateStr) > 0;
  return `${ret >= 0 ? "positive" : "negative"}${hasAmount ? " has-amount" : ""}`;
}

function getDayAmount(dateStr: string): number {
  const ts = dateTimestampMap.value.get(dateStr);
  return ts && editingFund.value?.schedule[String(ts)] ? editingFund.value.schedule[String(ts)] : 0;
}

async function openAddSchedule() {
  editingIdx.value = -1;
  editingFund.value = null;
  scheduleFundCode.value = "";
  scheduleHistory.value = [];
  scheduleCalendarDate.value = new Date();
  showScheduleDialog.value = true;
}

async function openSchedule(idx: number) {
  editingIdx.value = idx;
  const src = investments.value[idx];
  editingFund.value = { code: src.code, name: src.name, schedule: { ...(src.schedule || {}) } };
  scheduleFundCode.value = "";
  scheduleHistory.value = [];
  scheduleCalendarDate.value = new Date();
  showScheduleDialog.value = true;
  await fetchHistoryForSchedule();
}

async function loadFundForSchedule() {
  const code = scheduleFundCode.value;
  if (!code) return;
  const fund = funds.value.find((f) => f.id === code);
  if (!fund) return;
  editingIdx.value = investments.value.findIndex((i) => i.code === code);
  editingFund.value = {
    code: fund.id,
    name: fund.name,
    schedule: editingIdx.value >= 0 ? { ...(investments.value[editingIdx.value].schedule || {}) } : {},
  };
  scheduleHistory.value = [];
  scheduleCalendarDate.value = new Date();
  await fetchHistoryForSchedule();
}

async function fetchHistoryForSchedule() {
  if (!editingFund.value) return;
  scheduleHistoryLoading.value = true;
  try {
    const result = await invoke<FundHistory>("get_fund_history", {
      fundCode: editingFund.value.code,
      period: "1y",
    });
    console.log("历史数据:", result.data.length, "条");
    scheduleHistory.value = result.data;
  } catch (e) {
    console.error(e);
  } finally {
    scheduleHistoryLoading.value = false;
  }
}

async function closeSchedule() {
  if (editingFund.value) {
    const existing = investments.value.find((i) => i.code === editingFund.value!.code);
    if (existing) {
      existing.schedule = editingFund.value.schedule;
    } else {
      investments.value.push({ ...editingFund.value });
    }
    await saveInvestments();
  }
  showScheduleDialog.value = false;
  editingFund.value = null;
}

// ─── Day Input Popover ───

const showDayInput = ref(false);
const dayInputDate = ref("");
const dayInputAmount = ref(0);
const dayInputRef = ref<HTMLInputElement>();

function openDayInput(dateStr: string) {
  const ts = dateTimestampMap.value.get(dateStr);
  if (!ts) return;
  dayInputDate.value = dateStr;
  dayInputAmount.value = editingFund.value?.schedule[String(ts)] ?? 0;
  showDayInput.value = true;
  nextTick(() => dayInputRef.value?.focus());
}

async function confirmDayInput() {
  if (!editingFund.value) return;
  const ts = dateTimestampMap.value.get(dayInputDate.value);
  if (!ts) return;
  if (dayInputAmount.value > 0) {
    editingFund.value.schedule[String(ts)] = dayInputAmount.value;
  } else {
    delete editingFund.value.schedule[String(ts)];
  }
  showDayInput.value = false;
  await saveInvestments();
}

function hasHistory(code: string) {
  return cachedHistoryCodes.value.has(code);
}

function isFavorite(code: string) {
  return favorites.value.has(code);
}

async function toggleFavorite(code: string) {
  const had = favorites.value.has(code);
  if (had) {
    favorites.value.delete(code);
  } else {
    favorites.value.add(code);
  }
  try {
    const result = await invoke<string[]>("toggle_favorite", {
      fundCode: code,
    });
    favorites.value = new Set(result);
  } catch {
    if (had) {
      favorites.value.add(code);
    } else {
      favorites.value.delete(code);
    }
  }
}

const tableContainerRef = ref<HTMLElement>();
const tableBodyRef = ref<HTMLElement>();
const scrollTop = ref(0);
const containerHeight = ref(0);

const totalHeight = computed(() => filteredFunds.value.length * ROW_HEIGHT);

const startIndex = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / ROW_HEIGHT) - OVERSCAN),
);
const endIndex = computed(() =>
  Math.min(
    filteredFunds.value.length,
    Math.ceil((scrollTop.value + containerHeight.value) / ROW_HEIGHT) +
      OVERSCAN,
  ),
);
const visibleRows = computed(() =>
  filteredFunds.value.slice(startIndex.value, endIndex.value),
);
const offsetY = computed(() => startIndex.value * ROW_HEIGHT);

function onTableScroll(e: Event) {
  scrollTop.value = (e.target as HTMLElement).scrollTop;
}

function togglePeriodMenu() {
  showPeriodMenu.value = !showPeriodMenu.value;
}

function selectPeriod(value: string) {
  batchPeriod.value = value;
  showPeriodMenu.value = false;
}

function closePeriodMenu(e: MouseEvent) {
  const toggle = document.querySelector(".btn-arrow");
  if (toggle && !toggle.contains(e.target as Node)) {
    showPeriodMenu.value = false;
  }
}

function updateTableSize() {
  nextTick(() => {
    if (tableBodyRef.value) {
      containerHeight.value = tableBodyRef.value.clientHeight;
    }
  });
}

async function loadCachedHistoryCodes() {
  try {
    const codes = await invoke<string[]>("get_cached_history_codes");
    cachedHistoryCodes.value = new Set(codes);
  } catch (_e) {}
}

async function loadFavorites() {
  try {
    const codes = await invoke<string[]>("get_favorites");
    favorites.value = new Set(codes);
  } catch (_e) {}
}

const showInvestDetailDialog = ref(false);
const investDetailFund = ref<FundInvestment | null>(null);
const detailHistory = ref<HistoryPoint[]>([]);
const detailHistoryLoading = ref(false);
const detailCalendarDate = ref(new Date());

const investDetailTotal = computed(() => {
  if (!investDetailFund.value) return 0;
  return Object.values(investDetailFund.value.schedule).reduce((s, v) => s + v, 0);
});

const detailReturnMap = computed(() => {
  const sorted = [...detailHistory.value].sort((a, b) => a.timestamp - b.timestamp);
  const map = new Map<string, number>();
  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1].value;
    const cur = sorted[i].value;
    const dailyReturn = ((cur - prev) / (100 + prev)) * 100;
    map.set(formatDate(sorted[i].timestamp), dailyReturn);
  }
  return map;
});

const detailDateTimestampMap = computed(() => {
  const map = new Map<string, number>();
  for (const point of detailHistory.value) {
    map.set(formatDate(point.timestamp), point.timestamp);
  }
  return map;
});

const detailCalendarDays = computed(() => {
  const year = detailCalendarDate.value.getFullYear();
  const month = detailCalendarDate.value.getMonth();
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

function getDetailReturn(dateStr: string): number | null {
  return detailReturnMap.value.get(dateStr) ?? null;
}

function getDetailAmount(dateStr: string): number {
  if (!investDetailFund.value) return 0;
  const ts = detailDateTimestampMap.value.get(dateStr);
  return ts ? (investDetailFund.value.schedule[String(ts)] ?? 0) : 0;
}

function getDetailCellClass(dateStr: string) {
  const ret = getDetailReturn(dateStr);
  if (ret === null) return "";
  const hasAmount = getDetailAmount(dateStr) > 0;
  return `${ret >= 0 ? "positive" : "negative"}${hasAmount ? " has-amount" : ""}`;
}

function detailPrevMonth() {
  const d = new Date(detailCalendarDate.value);
  d.setMonth(d.getMonth() - 1);
  detailCalendarDate.value = d;
}
function detailNextMonth() {
  const d = new Date(detailCalendarDate.value);
  d.setMonth(d.getMonth() + 1);
  detailCalendarDate.value = d;
}

async function showInvestDetail(code: string) {
  const fund = investments.value.find((i) => i.code === code);
  if (!fund) return;
  investDetailFund.value = fund;
  detailCalendarDate.value = new Date();
  showInvestDetailDialog.value = true;
  detailHistoryLoading.value = true;
  try {
    const result = await invoke<FundHistory>("get_fund_history", {
      fundCode: fund.code,
      period: "1y",
    });
    detailHistory.value = result.data;
  } catch (e) {
    console.error(e);
  } finally {
    detailHistoryLoading.value = false;
  }
}

async function fetchAllFunds() {
  loading.value = true;
  progressText.value = "0/0";
  let lastUpdate = 0;
  const unlisten = await listen<CrawlProgress>("crawl-progress", (event) => {
    const now = Date.now();
    if (now - lastUpdate < 300) return;
    lastUpdate = now;
    const p = event.payload;
    const digits = String(p.total).length;
    progressText.value = `${String(p.current).padStart(digits, "0")}/${p.total}`;
  });
  try {
    const result = await invoke<FundItem[]>("fetch_all_funds");
    funds.value = result;
    cache = result;
    progressText.value = `完成，共 ${result.length} 只基金`;
    await nextTick();
    updateTableSize();
    loadCachedHistoryCodes();
    loadFavorites();
  } catch (e) {
    console.error(e);
    progressText.value = "获取失败";
  } finally {
    unlisten();
    loading.value = false;
  }
}

async function fetchAllHistory() {
  historyLoading.value = true;
  historyProgressText.value = "0/0";
  let lastUpdate = 0;
  const unlisten = await listen<HistoryCrawlProgress>(
    "history-crawl-progress",
    (event) => {
      const now = Date.now();
      if (now - lastUpdate < 300) return;
      lastUpdate = now;
      const p = event.payload;
      const digits = String(p.total).length;
      historyProgressText.value = `${String(p.current).padStart(digits, "0")}/${p.total}`;
    },
  );
  try {
    await invoke("fetch_all_history", { period: batchPeriod.value });
    historyProgressText.value = "全部完成";
    loadCachedHistoryCodes();
  } catch (e) {
    console.error(e);
    historyProgressText.value = "获取失败";
  } finally {
    unlisten();
    historyLoading.value = false;
  }
}

async function openHistoryDir() {
  try {
    await invoke("open_history_dir");
  } catch (_e) {}
}

function viewHistory(fundCode: string) {
  router.push(`/fund/${fundCode}`);
}
function viewSimulation(fundCode: string) {
  router.push(`/simulation/${fundCode}`);
}

let resizeObserver: ResizeObserver | null = null;

onMounted(async () => {
  document.addEventListener("mousedown", closePeriodMenu);
  if (cache) {
    funds.value = cache;
    loadFavorites();
    await loadAccount();
    await nextTick();
    updateTableSize();
    return;
  }
  try {
    const cached = await invoke<FundItem[]>("load_cached_funds");
    if (cached.length > 0) {
      funds.value = cached;
      cache = cached;
    }
  } catch (_e) {}
  loadCachedHistoryCodes();
  loadFavorites();
  await loadAccount();
  await nextTick();
  updateTableSize();

  if (tableContainerRef.value) {
    resizeObserver = new ResizeObserver(updateTableSize);
    resizeObserver.observe(tableContainerRef.value);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener("mousedown", closePeriodMenu);
  resizeObserver?.disconnect();
});
</script>

<style scoped>
.home {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* ─── Brand ─── */

.brand {
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 12px 16px 4px;
  flex-shrink: 0;
}
.brand h1 {
  font-family: var(--font-display);
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
}
.brand-o {
  color: var(--accent-gold);
}
.tagline {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 400;
}

/* ─── Toolbar ─── */

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  flex-shrink: 0;
  gap: 12px;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  flex: 0 0 auto;
}

/* ─── Filter Bar ─── */

.filter-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px 6px;
  flex-shrink: 0;
}
.filter-count {
  font-size: 12px;
  color: var(--text-secondary);
}

/* ─── Search ─── */

.search-wrap {
  position: relative;
  max-width: 260px;
  width: 100%;
}
.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 15px;
  height: 15px;
  color: var(--text-muted);
  pointer-events: none;
}
.search-input {
  width: 100%;
  height: 32px;
  padding: 0 10px 0 32px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-display);
  font-size: 13px;
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}
.search-input::placeholder {
  color: var(--text-muted);
}
.search-input:focus {
  border-color: var(--accent-gold);
  box-shadow: 0 0 0 1px var(--accent-gold);
}
.total-count {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  font-family: var(--font-display);
}
.filtered-count {
  font-size: 13px;
  color: var(--accent-gold);
  white-space: nowrap;
}

/* ─── View Tabs ─── */

.view-tabs {
  display: flex;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  overflow: hidden;
  flex-shrink: 0;
}
.view-tab {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: 500;
  padding: 0 12px;
  height: 26px;
  cursor: pointer;
  transition: all 0.15s ease;
  line-height: 1;
}
.view-tab:hover {
  color: var(--accent-gold);
}
.view-tab.active {
  background: var(--accent-gold);
  color: #0b0b0f;
}

/* ─── Buttons ─── */

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 14px;
  border-radius: var(--radius-sm);
  font-family: var(--font-body);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s ease;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-primary);
  line-height: 1;
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.btn-ghost {
  color: var(--text-secondary);
  border-color: transparent;
}
.btn-ghost:hover:not(:disabled) {
  color: var(--accent-gold);
  background: var(--accent-gold-muted);
}
.btn-outline {
  color: var(--accent-gold);
  border-color: var(--accent-gold);
  background: transparent;
}
.btn-outline:hover:not(:disabled) {
  background: var(--accent-gold-muted);
}
.btn-done {
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
  line-height: 1;
  color: var(--accent-gold);
  border: 1px solid var(--accent-gold);
  background: transparent;
}
.btn-done:hover {
  background: var(--accent-gold-muted);
}
.btn-primary {
  background: var(--accent-gold);
  border-color: var(--accent-gold);
  color: #0b0b0f;
}
.btn-primary:hover:not(:disabled) {
  background: var(--accent-gold-hover);
  border-color: var(--accent-gold-hover);
}

/* ─── Button Group & Dropdown ─── */

.btn-group {
  position: relative;
  display: flex;
}
.btn-group > .btn:first-child {
  border-radius: var(--radius-sm) 0 0 var(--radius-sm);
  border-right: none;
}
.btn-arrow {
  padding: 0 8px;
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
}
.dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  min-width: 120px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 4px;
  z-index: 100;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}
.dropdown-item {
  display: block;
  width: 100%;
  padding: 6px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: background 0.15s ease;
}
.dropdown-item:hover {
  background: var(--accent-gold-muted);
  color: var(--accent-gold);
}
.dropdown-item.active {
  color: var(--accent-gold);
  font-weight: 600;
  background: var(--accent-gold-muted);
}

/* ─── Spinner ─── */

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  flex-shrink: 0;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* ─── Table ─── */

.table-container {
  flex: 1;
  min-height: 0;
  padding: 0 16px 12px;
  display: flex;
  flex-direction: column;
  position: relative;
}

/* Header */
.table-header {
  flex-shrink: 0;
  padding-right: 6px;
}
.table-header .tr {
  border-bottom: 1px solid var(--border-subtle);
}
.th {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 0 8px;
  height: 36px;
  display: flex;
  align-items: center;
  white-space: nowrap;
}

/* Body */
.table-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
}
.virtual-sizer {
  position: relative;
}
.virtual-content {
  will-change: transform;
}

/* Row */
.tr {
  display: grid;
  grid-template-columns: 120px 1fr 200px 170px 72px;
  align-items: center;
  border-bottom: 1px solid var(--border-subtle);
  transition: background-color 0.15s ease;
}
.tr:hover {
  background: var(--accent-gold-muted);
  box-shadow: inset 3px 0 0 0 var(--accent-gold);
}

/* Cells */
.td {
  padding: 0 8px;
  display: flex;
  align-items: center;
  height: 100%;
  overflow: hidden;
}

.col-code {
  padding-left: 4px;
}
.col-action,
.col-status {
  justify-content: center;
}
.col-status {
  padding-right: 4px;
}

/* Code */
.code-text {
  font-family: var(--font-display);
  font-size: 13px;
  letter-spacing: 0.03em;
  color: var(--accent-gold);
  font-weight: 500;
}

/* Name */
.name-text {
  font-weight: 400;
  font-size: 13px;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Company */
.company-text {
  color: var(--text-secondary);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Star button */
.star-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0 6px 0 2px;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  color: var(--text-muted);
  transition:
    color 0.15s ease,
    transform 0.15s ease;
}
.star-btn:hover {
  color: var(--accent-gold);
  transform: scale(1.15);
}
.star-btn.starred {
  color: var(--accent-gold);
}

/* Action buttons */
.action-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-family: var(--font-body);
  font-size: 12px;
  cursor: pointer;
  padding: 0 6px;
  line-height: 1;
  transition: color 0.15s ease;
  white-space: nowrap;
}
.action-btn:hover {
  color: var(--accent-gold);
}
.action-divider {
  width: 1px;
  height: 12px;
  background: var(--border-default);
  flex-shrink: 0;
}

/* Coin button */
.coin-btn {
  background: none;
  border: none;
  color: var(--accent-gold);
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  transition: transform 0.15s ease;
}
.coin-btn:hover {
  transform: scale(1.15);
}

/* Status */
.status-cell {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-family: var(--font-display);
  font-size: 12px;
  line-height: 1;
}
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.status-dot.yes {
  background: var(--down-green);
  box-shadow: 0 0 4px rgba(39, 174, 96, 0.3);
}
.status-dot.no {
  background: var(--text-muted);
}

/* ─── Empty State ─── */

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-muted);
  gap: 6px;
}
.empty-icon {
  width: 40px;
  height: 40px;
  margin-bottom: 8px;
  opacity: 0.4;
}
.empty-state p {
  font-size: 13px;
}
.empty-hint {
  font-size: 12px;
  opacity: 0.7;
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

/* ─── Account Button ─── */

.brand-spacer {
  flex: 1;
}
.account-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}
.account-btn:hover {
  color: var(--accent-gold);
  border-color: var(--accent-gold);
  background: var(--accent-gold-muted);
}

/* ─── Account Dialog ─── */

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
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  animation: scaleIn 0.15s ease;
  display: flex;
  flex-direction: column;
  max-height: 80vh;
}
.account-dialog {
  width: 520px;
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
.dialog-header h3 {
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}
.dialog-close {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  padding: 0 4px;
  transition: color 0.15s ease;
}
.dialog-close:hover {
  color: var(--text-primary);
}
.dialog-body {
  padding: 14px 18px;
  overflow-y: auto;
  flex: 1;
}
.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 12px 18px;
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

/* Investment list (Account Dialog) */
.investment-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}
.investment-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
}
.inv-info {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.inv-code {
  font-family: var(--font-display);
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-gold);
  flex-shrink: 0;
}
.inv-name {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.inv-total {
  font-family: var(--font-display);
  font-size: 11px;
  color: var(--text-muted);
  margin-left: auto;
  flex-shrink: 0;
}
.inv-config-btn {
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--accent-gold);
  font-family: var(--font-body);
  font-size: 11px;
  padding: 2px 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}
.inv-config-btn:hover {
  background: var(--accent-gold-muted);
  border-color: var(--accent-gold);
}
.inv-remove {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  padding: 0 2px;
  transition: color 0.15s ease;
}
.inv-remove:hover {
  color: #e74c3c;
}
.add-investment-btn {
  width: 100%;
  justify-content: center;
}

/* Empty state */
.empty-investments {
  text-align: center;
  padding: 24px 0;
  color: var(--text-muted);
  font-size: 13px;
}

/* Total */
.total-investment {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-gold);
  margin-right: auto;
}

/* ─── Schedule Dialog ─── */

.schedule-dialog {
  width: 600px;
}

/* Fund select area */
.select-fund-area {
  display: flex;
  gap: 8px;
  padding: 8px 0;
}
.fund-select {
  flex: 1;
  height: 32px;
  padding: 0 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: 13px;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.fund-select:focus {
  border-color: var(--accent-gold);
}
.fund-select option {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

/* Schedule fund bar */
.schedule-fund-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}
.schedule-fund-code {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--accent-gold);
}

/* Calendar toolbar */
.calendar-toolbar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 4px 0 10px;
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

/* Calendar table */
.calendar-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}
.calendar-table th {
  padding: 5px 0;
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
  height: 58px;
}
.calendar-table td.is-empty {
  background: transparent;
  border-color: transparent;
}
.cal-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  min-height: 56px;
  padding: 3px 2px;
  cursor: pointer;
  transition: background 0.15s ease;
  position: relative;
}
.cal-cell:hover {
  background: var(--accent-gold-muted);
}
.cal-cell.positive {
  background: rgba(231, 76, 76, 0.03);
}
.cal-cell.negative {
  background: rgba(39, 174, 96, 0.03);
}
.cal-cell.positive:hover {
  background: rgba(231, 76, 76, 0.08);
}
.cal-cell.negative:hover {
  background: rgba(39, 174, 96, 0.08);
}
.cal-cell.has-amount {
  outline: 2px solid var(--accent-gold);
  outline-offset: -2px;
  border-radius: 2px;
}
.cal-day-num {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-display);
  line-height: 1.3;
}
.cal-return {
  font-family: var(--font-display);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: -0.01em;
  line-height: 1.2;
}
.cal-return.up {
  color: var(--up-red);
}
.cal-return.down {
  color: var(--down-green);
}
.cal-amount {
  font-family: var(--font-display);
  font-size: 9px;
  font-weight: 700;
  color: var(--accent-gold);
  line-height: 1;
  margin-top: 1px;
}

/* ─── Day Input Popover ─── */

.day-input-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}
.day-input-panel {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.5);
  width: 280px;
  animation: scaleIn 0.12s ease;
}
.day-input-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 8px;
  border-bottom: 1px solid var(--border-subtle);
}
.day-input-date {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}
.day-input-return {
  font-family: var(--font-display);
  font-size: 11px;
  font-weight: 600;
}
.day-input-return.up {
  color: var(--up-red);
}
.day-input-return.down {
  color: var(--down-green);
}
.day-input-body {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.day-input-body label {
  font-size: 12px;
  color: var(--text-secondary);
}
.day-input-field {
  width: 100%;
  height: 36px;
  padding: 0 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-display);
  font-size: 16px;
  outline: none;
  text-align: right;
  transition: border-color 0.2s ease;
  box-sizing: border-box;
}
.day-input-field:focus {
  border-color: var(--accent-gold);
  box-shadow: 0 0 0 1px var(--accent-gold);
}
.day-input-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px 14px 12px;
}

/* ─── Invest Detail Dialog ─── */

.invest-detail-dialog {
  width: 560px;
}
.invest-detail-total {
  text-align: right;
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 700;
  color: var(--accent-gold);
  padding: 8px 0 0;
}

/* Animations */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
@keyframes scaleIn {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
</style>
