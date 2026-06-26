<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
</script>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  color: #f6f6f6;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  background-attachment: fixed;
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
}

.el-table {
  --el-table-bg-color: rgba(255,255,255,0.03);
  --el-table-tr-bg-color: transparent;
  --el-table-header-bg-color: rgba(255,255,255,0.06);
  --el-table-row-hover-bg-color: rgba(255,255,255,0.08);
  --el-table-text-color: #e0e0e0;
  --el-table-header-text-color: #a0aec0;
  --el-table-border-color: rgba(255,255,255,0.08);
}

.el-table th.el-table__cell {
  background-color: rgba(255,255,255,0.06);
}

.el-button--primary {
  --el-button-bg-color: #667eea;
  --el-button-border-color: #667eea;
  --el-button-hover-bg-color: #5a6fd6;
  --el-button-hover-border-color: #5a6fd6;
}

.el-empty__description {
  color: #a0aec0;
}
</style>
