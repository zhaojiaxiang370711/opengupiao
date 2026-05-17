<template>
  <div class="neu-layout">
    <header class="neu-layout__header">
      <h1 class="neu-layout__title">美股看板</h1>
      <nav class="neu-layout__nav">
        <router-link v-for="item in navItems" :key="item.to" :to="item.to" class="neu-layout__nav-item">
          {{ item.label }}
        </router-link>
      </nav>
    </header>
    <main class="neu-layout__main">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
const navItems = [
  { to: '/', label: '看板' },
  { to: '/market', label: '行情' },
  { to: '/portfolio', label: '持仓' },
  { to: '/exchange', label: '汇率' },
  { to: '/assets', label: '资产' },
  { to: '/learn', label: '学习' },
]
</script>

<style scoped>
.neu-layout {
  min-height: 100vh;
  background: var(--neu-bg);
  color: var(--neu-text);
  font-family: 'Inter', 'PingFang SC', system-ui, sans-serif;
}

.neu-layout__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 20px 40px;
  background: var(--neu-bg);
  box-shadow:
    0 4px 8px var(--neu-shadow-dark),
    0 -2px 8px var(--neu-shadow-light);
  position: sticky;
  top: 0;
  z-index: 10;
}

.neu-layout__title {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 2px;
  color: var(--neu-primary);
}

.neu-layout__nav {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.neu-layout__nav-item {
  padding: 10px 20px;
  border-radius: var(--neu-radius-sm);
  text-decoration: none;
  color: var(--neu-text-dim);
  font-weight: 500;
  transition: all var(--neu-transition);
}

.neu-layout__nav-item:hover {
  color: var(--neu-text);
  box-shadow:
    3px 3px 6px var(--neu-shadow-dark),
    -3px -3px 6px var(--neu-shadow-light);
}

.neu-layout__nav-item.router-link-active {
  color: var(--neu-primary);
  box-shadow:
    inset 2px 2px 4px var(--neu-shadow-dark),
    inset -2px -2px 4px var(--neu-shadow-light);
}

.neu-layout__main {
  max-width: 1400px;
  margin: 0 auto;
  padding: 32px 40px;
}

@media (max-width: 760px) {
  .neu-layout__header {
    align-items: stretch;
    flex-direction: column;
    padding: 18px 20px;
  }

  .neu-layout__nav {
    justify-content: flex-start;
  }

  .neu-layout__nav-item {
    padding: 9px 14px;
  }

  .neu-layout__main {
    padding: 24px 20px;
  }
}
</style>
