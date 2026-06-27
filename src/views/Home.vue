<template>
  <div class="home">
    <div class="brand">
      <h1><span class="brand-o">o</span>Fund</h1>
      <span class="tagline">基金策略分析工具</span>
    </div>
    <div class="toolbar">
      <div class="toolbar-left">
        <el-input
          v-model="search"
          placeholder="搜索基金代码或名称..."
          clearable
          size="small"
          class="search-input"
        />
        <span class="total-count" v-if="funds.length"
          >共 {{ funds.length }} 只基金</span
        >
        <span
          class="filtered-count"
          v-if="search && filteredFunds.length !== funds.length"
        >
          / 筛选后 {{ filteredFunds.length }} 只
        </span>
      </div>
      <div class="toolbar-right">
        <el-button text size="small" @click="openHistoryDir">
          历史缓存
        </el-button>
        <el-dropdown
          split-button
          type="success"
          size="small"
          :disabled="historyLoading || !funds.length"
          @click="fetchAllHistory"
          @command="batchPeriod = $event"
        >
          {{
            historyLoading ? historyProgressText : `批量获取(${periodLabel})`
          }}
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                :class="{ active: batchPeriod === '1m' }"
                command="1m"
                >1个月</el-dropdown-item
              >
              <el-dropdown-item
                :class="{ active: batchPeriod === '3m' }"
                command="3m"
                >3个月</el-dropdown-item
              >
              <el-dropdown-item
                :class="{ active: batchPeriod === '6m' }"
                command="6m"
                >6个月</el-dropdown-item
              >
              <el-dropdown-item
                :class="{ active: batchPeriod === '1y' }"
                command="1y"
                >1年</el-dropdown-item
              >
              <el-dropdown-item
                :class="{ active: batchPeriod === 'all' }"
                command="all"
                >全部</el-dropdown-item
              >
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-button
          type="primary"
          @click="fetchAllFunds"
          :loading="loading"
          :disabled="loading"
        >
          {{ loading ? progressText : "更新基金列表" }}
        </el-button>
      </div>
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
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { h } from "vue";
import { ElButton } from "element-plus";

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

const columns = computed<any[]>(() => [
  {
    key: "id",
    title: "基金代码",
    width: 120,
    dataKey: "id",
    cellRenderer: ({ cellData }: any) =>
      h("span", { class: "code-cell" }, cellData),
  },
  {
    key: "name",
    title: "基金名称",
    width: 260,
    dataKey: "name",
    cellRenderer: ({ cellData }: any) =>
      h("span", { class: "name-cell" }, cellData),
  },
  {
    key: "company_name",
    title: "基金公司",
    width: 200,
    dataKey: "company_name",
    cellRenderer: ({ cellData }: any) =>
      h("span", { class: "company-cell" }, cellData),
  },
  {
    key: "action",
    title: "操作",
    width: 140,
    cellRenderer: ({ rowData }: any) =>
      h("span", { class: "action-group" }, [
        h(
          ElButton,
          {
            text: true,
            size: "small",
            onClick: () => viewHistory(rowData.id),
          },
          () => "走势",
        ),
        h(
          ElButton,
          {
            text: true,
            size: "small",
            onClick: () => viewSimulation(rowData.id),
          },
          () => "量化模拟",
        ),
      ]),
  },
  {
    key: "has_history",
    title: "已获取",
    width: 72,
    cellRenderer: ({ rowData }: any) => {
      const has = hasHistory(rowData.id);
      return h("span", { class: "status-cell" }, [
        h("span", { class: ["status-dot", has ? "yes" : "no"] }),
        has ? "有" : "无",
      ]);
    },
  },
]);

const tableRef = ref<HTMLElement | null>(null);
const tableHeight = ref(0);
const tableWidth = ref(0);

function updateTableSize() {
  nextTick(() => {
    if (!tableRef.value) return;
    const home = tableRef.value.parentElement;
    if (!home) return;
    const homeRect = home.getBoundingClientRect();
    const brandEl = home.querySelector(".brand") as HTMLElement | null;
    const toolbarEl = home.querySelector(".toolbar") as HTMLElement | null;
    let headerHeight = 0;
    if (brandEl) headerHeight += brandEl.offsetHeight;
    if (toolbarEl) headerHeight += toolbarEl.offsetHeight;
    tableHeight.value = homeRect.height - headerHeight;
    tableWidth.value = homeRect.width;
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

onMounted(async () => {
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
});

window.addEventListener("resize", updateTableSize);
onBeforeUnmount(() => window.removeEventListener("resize", updateTableSize));
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
  gap: 8px;
  flex: 1;
  min-width: 0;
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.search-input {
  max-width: 240px;
}
.search-input :deep(.el-input__inner) {
  font-family: var(--font-display);
  font-size: 13px;
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

/* ─── Table ─── */

.table-area {
  flex: 1;
  padding: 0 16px 12px;
  overflow: hidden;
}
.table-area :deep(.el-table-v2) {
  --el-table-v2-bg-color: var(--bg-surface);
  --el-table-v2-header-bg-color: var(--bg-surface);
  --el-table-v2-row-hover-bg-color: var(--accent-gold-muted);
  --el-table-v2-border-color: transparent;
}

/* Header */
.table-area :deep(.el-table-v2__header) {
  border-bottom: 1px solid var(--border-subtle);
}
.table-area :deep(.el-table-v2__header-cell) {
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 0 8px;
}
.table-area :deep(.el-table-v2__header-cell:first-child) {
  padding-left: 4px;
}

/* Rows */
.table-area :deep(.el-table-v2__row) {
  background-color: transparent;
  transition:
    box-shadow 0.2s ease,
    background-color 0.2s ease;
}
.table-area :deep(.el-table-v2__row:hover) {
  box-shadow: inset 3px 0 0 0 var(--accent-gold);
}
.table-area :deep(.el-table-v2__row-cell) {
  color: var(--text-primary);
  font-size: 13px;
  padding: 0 8px;
  height: 40px;
  border-bottom: 1px solid var(--border-subtle);
  transition: none;
}
.table-area :deep(.el-table-v2__row-cell:first-child) {
  padding-left: 4px;
}

/* Code column */
.code-cell {
  font-family: var(--font-display);
  font-size: 13px;
  letter-spacing: 0.03em;
  color: var(--accent-gold);
  font-weight: 500;
}

/* Name column */
.name-cell {
  font-weight: 400;
  font-size: 13px;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Company column */
.company-cell {
  color: var(--text-secondary);
  font-size: 12px;
}

/* Action column */
.table-area :deep(.el-table-v2__row-cell:nth-child(4)),
.table-area :deep(.el-table-v2__row-cell:nth-child(5)) {
  text-align: center;
}

/* ─── Action Group ─── */

.action-group {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.action-group .el-button.is-text {
  font-size: 12px;
  padding: 0 6px;
  white-space: nowrap;
}
.action-group .el-button.is-text + .el-button.is-text::before {
  content: "";
  width: 1px;
  height: 12px;
  background: var(--border-default);
  margin-right: 4px;
}

/* ─── Status Cell ─── */

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
</style>

<style>
.el-dropdown-menu__item.active {
  color: var(--accent-gold);
  font-weight: 600;
  background-color: var(--accent-gold-muted);
}
</style>
