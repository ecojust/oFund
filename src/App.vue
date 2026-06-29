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
