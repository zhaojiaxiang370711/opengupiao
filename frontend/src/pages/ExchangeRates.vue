<template>
  <div class="exchange-page">
    <section class="exchange-head neu-convex">
      <div>
        <h2>人民币 · 港币 · 美元</h2>
        <p>{{ latest.date }} 更新</p>
      </div>
      <div class="pair-tabs">
        <button
          v-for="pair in pairs"
          :key="pair.key"
          class="pair-tab"
          :class="{ 'pair-tab--active': activePairKey === pair.key }"
          type="button"
          @click="activePairKey = pair.key"
        >
          {{ pair.label }}
        </button>
      </div>
    </section>

    <section class="rate-grid">
      <NeuCard v-for="rate in rateCards" :key="rate.label" class="rate-card">
        <span>{{ rate.label }}</span>
        <strong>{{ rate.value }}</strong>
      </NeuCard>
    </section>

    <NeuCard class="chart-card">
      <div class="chart-card__head">
        <div>
          <h3>{{ activePair.label }} 汇率变化</h3>
          <p>近 60 个自然日</p>
        </div>
        <div class="trend-pill" :class="trendClass">
          {{ trendLabel }}
        </div>
      </div>
      <EChartsLine :points="chartPoints" :accent="trendAccent" />
      <div class="chart-foot">
        <span>{{ chartPoints.length }} 点</span>
        <span v-if="errorMessage" class="error-text">{{ errorMessage }}</span>
        <span v-else>{{ isLoading ? '同步中' : '已同步' }}</span>
      </div>
    </NeuCard>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import NeuCard from '../components/NeuCard.vue'
import EChartsLine from '../components/EChartsLine.vue'
import {
  defaultExchangeRates,
  fetchLatestRates,
  fetchRateHistory,
  formatCurrency,
  pairRate,
} from '../composables/useExchangeRates'
import type { CurrencyCode, ExchangeRatePoint, ExchangeRates } from '../types'

interface PairOption {
  key: string
  label: string
  from: CurrencyCode
  to: CurrencyCode
}

const pairs: PairOption[] = [
  { key: 'USD-CNY', label: '美元/人民币', from: 'USD', to: 'CNY' },
  { key: 'USD-HKD', label: '美元/港币', from: 'USD', to: 'HKD' },
  { key: 'HKD-CNY', label: '港币/人民币', from: 'HKD', to: 'CNY' },
  { key: 'CNY-HKD', label: '人民币/港币', from: 'CNY', to: 'HKD' },
]

const latest = ref<ExchangeRates>(defaultExchangeRates())
const history = ref<ExchangeRatePoint[]>([])
const activePairKey = ref('USD-CNY')
const isLoading = ref(false)
const errorMessage = ref('')

const activePair = computed(() =>
  pairs.find((pair) => pair.key === activePairKey.value) ?? pairs[0],
)

const rateCards = computed(() => [
  {
    label: '1 美元',
    value: `${formatCurrency(latest.value.rates.CNY, 'CNY')} / ${formatCurrency(latest.value.rates.HKD, 'HKD')}`,
  },
  {
    label: '1 港币',
    value: formatCurrency(pairRate('HKD', 'CNY', latest.value), 'CNY'),
  },
  {
    label: '1 人民币',
    value: `${formatCurrency(pairRate('CNY', 'USD', latest.value), 'USD')} / ${formatCurrency(pairRate('CNY', 'HKD', latest.value), 'HKD')}`,
  },
])

const chartPoints = computed(() =>
  history.value.map((point) => ({
    time: point.time,
    price: rateFromPoint(point, activePair.value.from, activePair.value.to),
  })),
)

const trend = computed(() => {
  if (chartPoints.value.length < 2) return 0
  const first = chartPoints.value[0].price
  const last = chartPoints.value[chartPoints.value.length - 1].price
  return first === 0 ? 0 : ((last - first) / first) * 100
})

const trendAccent = computed(() => {
  if (trend.value > 0) return 'up'
  if (trend.value < 0) return 'down'
  return 'flat'
})

const trendClass = computed(() => ({
  up: trendAccent.value === 'up',
  down: trendAccent.value === 'down',
}))

const trendLabel = computed(() =>
  `${trend.value >= 0 ? '+' : ''}${trend.value.toFixed(2)}%`,
)

function rateFromPoint(point: ExchangeRatePoint, from: CurrencyCode, to: CurrencyCode): number {
  const rates: ExchangeRates = {
    base: 'USD',
    date: point.date,
    rates: {
      USD: 1,
      CNY: point.usdCny,
      HKD: point.usdHkd,
    },
  }
  return pairRate(from, to, rates)
}

async function refreshRates() {
  isLoading.value = true
  errorMessage.value = ''

  try {
    const [nextLatest, nextHistory] = await Promise.all([
      fetchLatestRates(),
      fetchRateHistory(60),
    ])
    latest.value = nextLatest
    history.value = nextHistory
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '汇率同步失败'
  } finally {
    isLoading.value = false
  }
}

onMounted(refreshRates)
</script>

<style scoped>
.exchange-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.exchange-head,
.chart-card__head,
.chart-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.exchange-head {
  padding: 20px;
}

.exchange-head h2,
.chart-card__head h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 800;
}

.exchange-head p,
.chart-card__head p,
.chart-foot {
  margin: 6px 0 0;
  color: var(--neu-text-dim);
  font-size: 13px;
}

.pair-tabs {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.pair-tab {
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 10px 14px;
  background: var(--neu-bg);
  color: var(--neu-text-dim);
  font-weight: 700;
  cursor: pointer;
}

.pair-tab--active {
  color: #fff;
  background: var(--neu-primary);
}

.rate-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 18px;
}

.rate-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.rate-card span {
  color: var(--neu-text-dim);
  font-size: 13px;
}

.rate-card strong {
  font-size: 22px;
}

.chart-card__head {
  margin-bottom: 18px;
}

.trend-pill {
  flex-shrink: 0;
  border-radius: 999px;
  padding: 8px 14px;
  color: var(--neu-text);
  background: var(--neu-bg-alt);
  font-weight: 800;
}

.trend-pill.up {
  color: var(--neu-success);
}

.trend-pill.down {
  color: var(--neu-danger);
}

.chart-foot {
  min-height: 18px;
  margin-top: 12px;
}

.error-text {
  color: var(--neu-danger);
}

@media (max-width: 760px) {
  .exchange-head,
  .chart-card__head {
    align-items: stretch;
    flex-direction: column;
  }

  .pair-tabs {
    justify-content: flex-start;
  }
}
</style>
