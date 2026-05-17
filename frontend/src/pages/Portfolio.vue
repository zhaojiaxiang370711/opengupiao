<template>
  <div class="portfolio">
    <div class="portfolio-grid">
      <div class="portfolio-main">
        <NeuCard>
          <h3 class="section-title">持仓概览</h3>
          <div class="holdings-empty" v-if="!store.holdings.length">
            <p>暂无持仓数据</p>
            <NeuButton variant="primary">添加持仓</NeuButton>
          </div>
          <div v-else class="holdings-list">
            <div v-for="h in store.holdings" :key="h.id" class="holding-row neu-concave">
              <span class="hr-symbol">{{ h.symbol }}</span>
              <span class="hr-qty">{{ h.quantity }} 股</span>
              <span class="hr-cost">成本 ${{ h.avg_cost.toFixed(2) }}</span>
            </div>
          </div>
        </NeuCard>
      </div>
      <div class="portfolio-side">
        <WatchlistPanel :items="store.watchlist" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import NeuCard from '../components/NeuCard.vue'
import NeuButton from '../components/NeuButton.vue'
import WatchlistPanel from '../components/WatchlistPanel.vue'
import { usePortfolioStore } from '../stores/portfolio'

const store = usePortfolioStore()

onMounted(async () => {
  await store.fetchPortfolio()
})
</script>

<style scoped>
.portfolio {
  max-width: 100%;
}

.portfolio-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 24px;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 17px;
  font-weight: 600;
}

.holdings-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--neu-text-dim);
}

.holdings-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.holding-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
}

.hr-symbol {
  font-weight: 700;
  color: var(--neu-primary);
}

.hr-qty {
  color: var(--neu-text);
  font-weight: 500;
}

.hr-cost {
  color: var(--neu-text-dim);
  font-size: 14px;
}

@media (max-width: 768px) {
  .portfolio-grid {
    grid-template-columns: 1fr;
  }
}
</style>
