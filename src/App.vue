<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
</script>

<style>
:root {
  --bg-deep: #0B0B0F;
  --bg-surface: #141418;
  --bg-elevated: #1A1A20;
  --text-primary: #EBEBEB;
  --text-secondary: #8B8B8B;
  --text-muted: #5A5A5A;
  --accent-gold: #D4A84B;
  --accent-gold-hover: #C49535;
  --accent-gold-muted: rgba(212, 168, 75, 0.12);
  --accent-gold-glow: rgba(212, 168, 75, 0.25);
  --up-red: #E74C4C;
  --down-green: #27AE60;
  --border-subtle: #232326;
  --border-default: #2A2A2E;
  --font-display: 'SF Mono', 'Cascadia Code', 'Menlo', 'Consolas', monospace;
  --font-body: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Noto Sans SC', 'PingFang SC', 'Microsoft YaHei', 'Helvetica Neue', Arial, sans-serif;
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;

  font-family: var(--font-body);
  font-size: 14px;
  line-height: 1.5;
  font-weight: 400;
  color: var(--text-primary);
  background: var(--bg-deep);
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

::selection {
  background: var(--accent-gold-muted);
  color: var(--accent-gold);
}

/* ─── Element Plus Overrides ─── */

.el-button--primary {
  --el-button-bg-color: var(--accent-gold);
  --el-button-border-color: var(--accent-gold);
  --el-button-text-color: #0B0B0F;
  --el-button-hover-bg-color: var(--accent-gold-hover);
  --el-button-hover-border-color: var(--accent-gold-hover);
  --el-button-hover-text-color: #0B0B0F;
  --el-button-active-bg-color: var(--accent-gold-hover);
  --el-button-active-border-color: var(--accent-gold-hover);
}

.el-button--primary.is-disabled {
  --el-button-disabled-bg-color: var(--accent-gold-muted);
  --el-button-disabled-border-color: transparent;
  --el-button-disabled-text-color: var(--text-muted);
}

.el-button--success {
  --el-button-bg-color: transparent;
  --el-button-border-color: var(--accent-gold);
  --el-button-text-color: var(--accent-gold);
  --el-button-hover-bg-color: var(--accent-gold-muted);
  --el-button-hover-border-color: var(--accent-gold);
  --el-button-hover-text-color: var(--accent-gold);
}

.el-button--success.is-disabled {
  --el-button-disabled-bg-color: transparent;
  --el-button-disabled-border-color: var(--border-default);
  --el-button-disabled-text-color: var(--text-muted);
}

.el-button--default,
.el-button--info {
  --el-button-bg-color: transparent;
  --el-button-border-color: var(--border-default);
  --el-button-text-color: var(--text-secondary);
  --el-button-hover-bg-color: rgba(255,255,255,0.04);
  --el-button-hover-border-color: var(--accent-gold);
  --el-button-hover-text-color: var(--accent-gold);
}

.el-button {
  border-radius: var(--radius-sm);
  font-family: var(--font-body);
  font-weight: 500;
}

.el-button.is-text {
  color: var(--text-secondary);
}
.el-button.is-text:hover {
  color: var(--accent-gold);
  background: var(--accent-gold-muted);
}

.el-input {
  --el-input-bg-color: var(--bg-surface);
  --el-input-border-color: var(--border-default);
  --el-input-hover-border-color: var(--accent-gold);
  --el-input-focus-border-color: var(--accent-gold);
  --el-input-text-color: var(--text-primary);
  --el-input-placeholder-color: var(--text-muted);
  --el-input-focus-text-color: var(--text-primary);
}

.el-input__wrapper {
  border-radius: var(--radius-sm);
  box-shadow: 0 0 0 1px var(--border-default) inset;
  transition: box-shadow 0.2s ease;
}
.el-input__wrapper:hover {
  box-shadow: 0 0 0 1px var(--accent-gold) inset;
}
.el-input__wrapper.is-focus {
  box-shadow: 0 0 0 1px var(--accent-gold) inset;
}

.el-table {
  --el-table-bg-color: transparent;
  --el-table-tr-bg-color: transparent;
  --el-table-header-bg-color: rgba(255,255,255,0.02);
  --el-table-row-hover-bg-color: var(--accent-gold-muted);
  --el-table-text-color: var(--text-primary);
  --el-table-header-text-color: var(--text-secondary);
  --el-table-border-color: var(--border-subtle);
}

.el-table th.el-table__cell {
  background-color: rgba(255,255,255,0.02);
  font-family: var(--font-body);
  font-weight: 500;
  font-size: 12px;
  text-transform: none;
  letter-spacing: 0;
}

.el-table__cell {
  border-bottom: 1px solid var(--border-subtle);
}

.el-table-v2 {
  --el-table-v2-bg-color: var(--bg-surface);
}

.el-dropdown-menu {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 4px;
}
.el-dropdown-menu__item {
  color: var(--text-primary);
  font-size: 13px;
  border-radius: var(--radius-sm);
  padding: 6px 12px;
}
.el-dropdown-menu__item:hover {
  background: var(--accent-gold-muted);
  color: var(--accent-gold);
}

.el-radio-button {
  --el-radio-button-checked-bg-color: var(--accent-gold);
  --el-radio-button-checked-text-color: #0B0B0F;
  --el-radio-button-checked-border-color: var(--accent-gold);
  --el-radio-button-bg-color: transparent;
  --el-radio-button-text-color: var(--text-secondary);
  --el-radio-button-border-color: var(--border-default);
  --el-radio-button-hover-bg-color: var(--accent-gold-muted);
  --el-radio-button-hover-text-color: var(--accent-gold);
}

.el-radio-button__inner {
  border-color: var(--border-default);
  font-family: var(--font-body);
  font-size: 12px;
  padding: 4px 12px;
}

.el-empty__description {
  color: var(--text-muted);
}

.el-loading-mask {
  background: rgba(11, 11, 15, 0.7);
}

.el-calendar {
  --el-calendar-border: var(--border-subtle);
  --el-calendar-header-border: var(--border-subtle);
  --el-calendar-selected-bg-color: var(--accent-gold-muted);
  --el-calendar-cell-width: auto;
}

.el-calendar-table td {
  border-color: var(--border-subtle);
  text-align: center;
}
.el-calendar-table td.is-today {
  background: var(--accent-gold-muted);
}
.el-calendar-table td.is-selected {
  background: transparent;
}
.el-calendar-table th {
  border-color: var(--border-subtle);
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 12px;
}
.el-calendar-day {
  height: auto;
  padding: 4px;
}

/* ─── Scrollbar ─── */

::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

/* ─── Reduced Motion ─── */

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* ─── Focus ─── */

*:focus-visible {
  outline: 1px solid var(--accent-gold);
  outline-offset: 2px;
}
</style>
