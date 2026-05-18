<template>
  <div class="assets-page">
    <section class="assets-head neu-convex">
      <div>
        <h2>我的资产</h2>
        <p>{{ latest.date }} 汇率 · 股票按实时价格估值</p>
      </div>
      <div class="currency-tabs">
        <button
          v-for="currency in currencies"
          :key="currency"
          class="currency-tab"
          :class="{ 'currency-tab--active': displayCurrency === currency }"
          type="button"
          @click="displayCurrency = currency"
        >
          {{ currencyName(currency) }}
        </button>
      </div>
    </section>

    <section class="summary-grid">
      <NeuCard class="summary-card summary-card--primary">
        <span>总资产</span>
        <strong>{{ money(totalDisplay) }}</strong>
        <small :class="pnlClass">{{ pnlLabel }}</small>
      </NeuCard>
      <NeuCard class="summary-card">
        <span>股票市值</span>
        <strong>{{ money(stockDisplay) }}</strong>
        <small>{{ store.holdings.length }} 个持仓</small>
      </NeuCard>
      <NeuCard class="summary-card">
        <span>现金</span>
        <strong>{{ money(cashDisplay) }}</strong>
        <small>人民币 / 美元 / 港币</small>
      </NeuCard>
      <NeuCard class="summary-card">
        <span>今日波动</span>
        <strong :class="dayClass">{{ money(dayChangeDisplay) }}</strong>
        <small>按持仓股数估算</small>
      </NeuCard>
    </section>

    <section class="assets-grid">
      <NeuCard class="holdings-card">
        <div class="section-head">
          <h3>持仓</h3>
          <button class="plain-action" type="button" @click="refreshAssets">刷新估值</button>
        </div>
        <div v-if="!holdingRows.length" class="empty-state">暂无持仓数据</div>
        <div v-else class="holding-list">
          <div v-for="row in holdingRows" :key="row.symbol" class="holding-row neu-concave">
            <div>
              <strong>{{ row.symbol }}</strong>
              <span>{{ row.quantity }} 股 · 成本 ${{ row.avgCost.toFixed(2) }}</span>
            </div>
            <div class="holding-values">
              <strong>{{ money(row.valueDisplay) }}</strong>
              <span :class="{ up: row.pnlDisplay >= 0, down: row.pnlDisplay < 0 }">
                {{ row.pnlDisplay >= 0 ? '+' : '' }}{{ money(row.pnlDisplay) }}
              </span>
            </div>
          </div>
        </div>
      </NeuCard>

      <NeuCard class="cash-card">
        <div class="section-head">
          <h3>现金</h3>
          <span>{{ money(cashDisplay) }}</span>
        </div>
        <label v-for="currency in currencies" :key="currency" class="cash-input">
          <span>{{ currencyName(currency) }}</span>
          <input v-model.number="cashBalances[currency]" type="number" min="0" step="0.01" />
        </label>
      </NeuCard>
    </section>

    <NeuCard class="asset-chart">
      <div class="section-head">
        <div>
          <h3>财产变化</h3>
          <p>按当前选择币种折算</p>
        </div>
        <span>{{ assetChartPoints.length }} 点</span>
      </div>
      <EChartsLine :points="assetChartPoints" :accent="assetTrend" />
      <div class="chart-note" v-if="assetChartPoints.length < 2">
        多刷新几次后会形成资产变化曲线
      </div>
    </NeuCard>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import NeuCard from '../components/NeuCard.vue'
import EChartsLine from '../components/EChartsLine.vue'
import {
  convertCurrency,
  currencyName,
  defaultExchangeRates,
  fetchLatestRates,
  formatCurrency,
} from '../composables/useExchangeRates'
import { useMarketStore } from '../stores/market'
import { usePortfolioStore } from '../stores/portfolio'
import type { CashBalances, CurrencyCode, ExchangeRates } from '../types'

interface AssetSnapshot {
  time: number
  totalUsd: number
}

const CASH_KEY = 'aaagupiao.cashBalances.v1'
const SNAPSHOT_KEY = 'aaagupiao.assetSnapshots.v1'
const currencies: CurrencyCode[] = ['CNY', 'USD', 'HKD']
const store = usePortfolioStore()
const market = useMarketStore()
const latest = ref<ExchangeRates>(defaultExchangeRates())
const displayCurrency = ref<CurrencyCode>('CNY')
const cashBalances = ref<CashBalances>(loadCash())
const snapshots = ref<AssetSnapshot[]>(loadSnapshots())

const holdingRows = computed(() =>
  store.holdings.map((holding) => {
    const quote = market.quotes.get(holding.symbol)
    const price = quote?.price ?? holding.avg_cost
    const valueUsd = price * holding.quantity
    const costUsd = holding.avg_cost * holding.quantity
    const pnlUsd = valueUsd - costUsd

    return {
      symbol: holding.symbol,
      quantity: holding.quantity,
      avgCost: holding.avg_cost,
      valueUsd,
      costUsd,
      pnlUsd,
      dayChangeUsd: (quote?.change ?? 0) * holding.quantity,
      valueDisplay: usdToDisplay(valueUsd),
      pnlDisplay: usdToDisplay(pnlUsd),
    }
  }),
)

const stockUsd = computed(() =>
  holdingRows.value.reduce((sum, row) => sum + row.valueUsd, 0),
)
const costUsd = computed(() =>
  holdingRows.value.reduce((sum, row) => sum + row.costUsd, 0),
)
const pnlUsd = computed(() => stockUsd.value - costUsd.value)
const cashUsd = computed(() =>
  convertCurrency(cashBalances.value.USD, 'USD', 'USD', latest.value)
  + convertCurrency(cashBalances.value.CNY, 'CNY', 'USD', latest.value)
  + convertCurrency(cashBalances.value.HKD, 'HKD', 'USD', latest.value),
)
const totalUsd = computed(() => stockUsd.value + cashUsd.value)
const dayChangeUsd = computed(() =>
  holdingRows.value.reduce((sum, row) => sum + row.dayChangeUsd, 0),
)

const totalDisplay = computed(() => usdToDisplay(totalUsd.value))
const stockDisplay = computed(() => usdToDisplay(stockUsd.value))
const cashDisplay = computed(() => usdToDisplay(cashUsd.value))
const dayChangeDisplay = computed(() => usdToDisplay(dayChangeUsd.value))
const pnlDisplay = computed(() => usdToDisplay(pnlUsd.value))
const pnlClass = computed(() => ({
  up: pnlDisplay.value >= 0,
  down: pnlDisplay.value < 0,
}))
const dayClass = computed(() => ({
  up: dayChangeDisplay.value >= 0,
  down: dayChangeDisplay.value < 0,
}))
const pnlLabel = computed(() =>
  `持仓盈亏 ${pnlDisplay.value >= 0 ? '+' : ''}${money(pnlDisplay.value)}`,
)
const assetChartPoints = computed(() =>
  snapshots.value.map((snapshot) => ({
    time: snapshot.time,
    price: usdToDisplay(snapshot.totalUsd),
  })),
)
const assetTrend = computed(() => {
  if (assetChartPoints.value.length < 2) return 'flat'
  const first = assetChartPoints.value[0].price
  const last = assetChartPoints.value[assetChartPoints.value.length - 1].price
  if (last > first) return 'up'
  if (last < first) return 'down'
  return 'flat'
})

function usdToDisplay(value: number): number {
  return convertCurrency(value, 'USD', displayCurrency.value, latest.value)
}

function money(value: number): string {
  return formatCurrency(value, displayCurrency.value)
}

function loadCash(): CashBalances {
  try {
    const raw = localStorage.getItem(CASH_KEY)
    if (!raw) return { USD: 0, CNY: 0, HKD: 0 }
    return { USD: 0, CNY: 0, HKD: 0, ...JSON.parse(raw) }
  } catch {
    return { USD: 0, CNY: 0, HKD: 0 }
  }
}

function loadSnapshots(): AssetSnapshot[] {
  try {
    const raw = localStorage.getItem(SNAPSHOT_KEY)
    if (!raw) return []
    return (JSON.parse(raw) as AssetSnapshot[])
      .filter((snapshot) => Number.isFinite(snapshot.totalUsd) && snapshot.totalUsd > 0)
      .slice(-500)
  } catch {
    return []
  }
}

function persistCash() {
  localStorage.setItem(CASH_KEY, JSON.stringify(cashBalances.value))
}

function recordSnapshot() {
  if (totalUsd.value <= 0) return
  const current = snapshots.value
  const last = current[current.length - 1]
  const next = { time: Date.now(), totalUsd: totalUsd.value }
  const merged = last && next.time - last.time < 60_000
    ? [...current.slice(0, -1), next]
    : [...current, next]
  snapshots.value = merged.slice(-500)
  localStorage.setItem(SNAPSHOT_KEY, JSON.stringify(snapshots.value))
}

async function refreshAssets() {
  const nextRates = await fetchLatestRates().catch(() => latest.value)
  latest.value = nextRates
  await store.fetchPortfolio()
  await Promise.all(store.holdings.map((holding) => market.fetchQuote(holding.symbol)))
  recordSnapshot()
}

watch(cashBalances, () => {
  persistCash()
  recordSnapshot()
}, { deep: true })

onMounted(refreshAssets)
</script>

<style scoped>
.assets-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.assets-head,
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.assets-head {
  padding: 20px;
}

.assets-head h2,
.section-head h3 {
  margin: 0;
  font-size: 22px;
  font-weight: 800;
}

.assets-head p,
.section-head p,
.summary-card span,
.summary-card small,
.holding-row span,
.chart-note {
  margin: 6px 0 0;
  color: var(--neu-text-dim);
  font-size: 13px;
}

.currency-tabs {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.currency-tab,
.plain-action {
  border: none;
  border-radius: var(--neu-radius-sm);
  background: var(--neu-bg);
  color: var(--neu-text-dim);
  cursor: pointer;
  font-weight: 800;
}

.currency-tab {
  padding: 10px 16px;
}

.currency-tab--active {
  color: #fff;
  background: var(--neu-primary);
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 18px;
}

.summary-card {
  display: flex;
  min-height: 126px;
  flex-direction: column;
  justify-content: space-between;
}

.summary-card strong {
  font-size: 26px;
  letter-spacing: 0;
}

.summary-card--primary strong {
  font-size: 32px;
}

.up {
  color: var(--neu-success) !important;
}

.down {
  color: var(--neu-danger) !important;
}

.assets-grid {
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(260px, 1fr);
  gap: 24px;
}

.plain-action {
  padding: 9px 12px;
}

.empty-state {
  padding: 42px 0;
  color: var(--neu-text-dim);
  text-align: center;
}

.holding-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 18px;
}

.holding-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 16px 18px;
}

.holding-row > div {
  display: flex;
  min-width: 0;
  flex-direction: column;
}

.holding-values {
  align-items: flex-end;
  text-align: right;
}

.cash-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.cash-input {
  display: grid;
  grid-template-columns: 80px minmax(0, 1fr);
  gap: 12px;
  align-items: center;
}

.cash-input span {
  color: var(--neu-text-dim);
  font-weight: 700;
}

.cash-input input {
  width: 100%;
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 12px 14px;
  background: var(--neu-bg);
  box-shadow:
    inset 3px 3px 6px var(--neu-shadow-dark),
    inset -3px -3px 6px var(--neu-shadow-light);
  color: var(--neu-text);
  font-size: 15px;
  outline: none;
}

.asset-chart .section-head {
  margin-bottom: 18px;
}

.chart-note {
  margin-top: 12px;
}

@media (max-width: 880px) {
  .assets-head,
  .section-head,
  .holding-row {
    align-items: stretch;
    flex-direction: column;
  }

  .currency-tabs {
    justify-content: flex-start;
  }

  .assets-grid {
    grid-template-columns: 1fr;
  }

  .holding-values {
    align-items: flex-start;
    text-align: left;
  }
}
</style>
