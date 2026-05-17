<template>
  <NeuCard>
    <h3 class="panel-title">自选股</h3>
    <div class="watchlist" v-if="items.length">
      <div v-for="item in items" :key="item.id" class="watchlist-item neu-concave">
        <span class="watchlist-symbol">{{ item.symbol }}</span>
        <span class="watchlist-badge">自选</span>
      </div>
    </div>
    <div v-else class="watchlist-empty">
      <p>暂无自选股</p>
      <NeuButton variant="primary">添加股票</NeuButton>
    </div>
  </NeuCard>
</template>

<script setup lang="ts">
import type { WatchlistItem } from '../types'
import NeuCard from './NeuCard.vue'
import NeuButton from './NeuButton.vue'

defineProps<{ items: WatchlistItem[] }>()
</script>

<style scoped>
.panel-title {
  margin: 0 0 16px 0;
  font-size: 17px;
  font-weight: 600;
}

.watchlist {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.watchlist-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
}

.watchlist-symbol {
  font-weight: 700;
  font-size: 16px;
  color: var(--neu-primary);
}

.watchlist-badge {
  font-size: 12px;
  color: var(--neu-text-dim);
  padding: 4px 10px;
  border-radius: 8px;
  background: var(--neu-bg);
  box-shadow:
    inset 1px 1px 2px var(--neu-shadow-dark),
    inset -1px -1px 2px var(--neu-shadow-light);
}

.watchlist-empty {
  text-align: center;
  padding: 32px 0;
  color: var(--neu-text-dim);
}
</style>
