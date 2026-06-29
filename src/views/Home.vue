<template>
  <div class="home">
    <div class="brand">
      <h1><span class="brand-o">o</span>Fund</h1>
      <span class="tagline">基金策略分析工具</span>
    </div>

    <div class="toolbar">
      <div class="toolbar-left">
        <div class="search-wrap">
          <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
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
        <span v-if="search && filteredFunds.length !== funds.length" class="filtered-count">
          / {{ filteredFunds.length.toLocaleString() }} 只
        </span>
      </div>
      <div class="toolbar-right">
        <button class="btn btn-ghost" @click="openHistoryDir">历史缓存</button>

        <div class="btn-group">
          <button
            class="btn btn-outline"
            :disabled="historyLoading || !funds.length"
            @click="fetchAllHistory"
          >
            <span v-if="historyLoading" class="spinner"></span>
            {{ historyLoading ? historyProgressText : `批量获取(${periodLabel})` }}
          </button>
          <button
            class="btn btn-outline btn-arrow"
            :disabled="historyLoading || !funds.length"
            @click="togglePeriodMenu"
            ref="periodToggleRef"
          >
            <svg viewBox="0 0 24 24" width="10" height="10" fill="currentColor">
              <path d="M7 10l5 5 5-5z"/>
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
            <div class="virtual-content" :style="{ transform: `translateY(${offsetY}px)` }">
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
                  <button class="action-btn" @click="viewHistory(row.id)">走势</button>
                  <span class="action-divider"></span>
                  <button class="action-btn" @click="viewSimulation(row.id)">量化模拟</button>
                </div>
                <div class="td col-status">
                  <span class="status-cell">
                    <span class="status-dot" :class="hasHistory(row.id) ? 'yes' : 'no'"></span>
                    {{ hasHistory(row.id) ? '有' : '无' }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </template>
        <div v-else class="empty-state">
          <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M9 3v18M15 3v18M3 9h18M3 15h18"/>
          </svg>
          <p>{{ search ? '没有匹配的基金' : '暂无数据' }}</p>
          <p v-if="!search" class="empty-hint">点击「更新基金列表」获取数据</p>
        </div>
      </div>

      <div v-if="loading" class="loading-overlay">
        <div class="loading-spinner"></div>
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
const search = ref("");

const funds = ref<FundItem[]>([]);
const cachedHistoryCodes = ref(new Set<string>());

const filteredFunds = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return funds.value;
  return funds.value.filter(
    (f) => f.id.toLowerCase().includes(q) || f.name.toLowerCase().includes(q),
  );
});

function hasHistory(code: string) {
  return cachedHistoryCodes.value.has(code);
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
    Math.ceil((scrollTop.value + containerHeight.value) / ROW_HEIGHT) + OVERSCAN,
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
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
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
.btn-primary {
  background: var(--accent-gold);
  border-color: var(--accent-gold);
  color: #0B0B0F;
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
  to { transform: rotate(360deg); }
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
  grid-template-columns: 120px 1fr 200px 140px 72px;
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
.col-action, .col-status {
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
</style>
