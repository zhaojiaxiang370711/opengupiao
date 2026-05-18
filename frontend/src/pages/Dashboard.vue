<template>
  <div class="dashboard">
    <section class="dashboard-overview neu-convex">
      <div class="dashboard-status">
        <span class="status-label">首页同步</span>
        <strong>{{ isRefreshing ? '后台刷新中' : '已就绪' }}</strong>
        <button class="refresh-button" type="button" :disabled="isRefreshing" @click="refreshAll">刷新</button>
      </div>

      <div class="clock-grid">
        <div class="clock-card neu-concave">
          <span>中国时间</span>
          <strong>{{ chinaClock.time }}</strong>
          <small>{{ chinaClock.date }} · 北京/上海</small>
        </div>
        <div class="clock-card neu-concave">
          <span>美国时间</span>
          <strong>{{ usClock.time }}</strong>
          <small>{{ usClock.date }} · 美东/纽约</small>
        </div>
      </div>
    </section>

    <section class="dashboard-indices">
      <div v-for="row in indexRows" :key="row.symbol" class="index-card neu-concave">
        <span class="index-name">{{ row.label }}</span>
        <strong class="index-price" :class="row.trend">
          {{ row.quote ? row.quote.price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) : '--' }}
        </strong>
        <span v-if="row.quote" class="index-change" :class="row.trend">
          {{ row.quote.change >= 0 ? '+' : '' }}{{ row.quote.change.toFixed(2) }}
          <small>({{ row.quote.change_percent >= 0 ? '+' : '' }}{{ row.quote.change_percent.toFixed(2) }}%)</small>
        </span>
        <span v-else class="index-change muted">同步中</span>
      </div>
    </section>

    <div class="dashboard-grid">
      <NeuCard v-for="row in quoteRows" :key="row.symbol" class="quote-card">
        <div class="quote-header">
          <span class="quote-symbol">{{ row.symbol }}</span>
          <span class="quote-price" :class="{ muted: !row.quote }">
            {{ row.quote ? `$${row.quote.price.toFixed(2)}` : '--' }}
          </span>
        </div>
        <div v-if="row.quote" class="quote-change" :class="row.quote.change >= 0 ? 'up' : 'down'">
          <span>{{ row.quote.change >= 0 ? '+' : '' }}{{ row.quote.change.toFixed(2) }}</span>
          <span class="quote-percent">
            ({{ row.quote.change_percent >= 0 ? '+' : '' }}{{ row.quote.change_percent.toFixed(2) }}%)
          </span>
        </div>
        <div v-else class="quote-change muted">
          {{ row.loading ? '同步中' : '等待同步' }}
        </div>
        <div class="quote-volume">成交量: {{ row.quote ? formatVolume(row.quote.volume) : '--' }}</div>
        <div class="quote-updated" :class="{ refreshing: row.loading }">
          {{ row.loading ? '正在更新' : formatQuoteTime(row.quote?.timestamp) }}
        </div>
      </NeuCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import NeuCard from '../components/NeuCard.vue'
import { useMarketStore } from '../stores/market'
import type { Quote } from '../types'

interface ClockParts {
  time: string
  date: string
}

interface QuoteRow {
  symbol: string
  quote: Quote | undefined
  loading: boolean
}

interface IndexDef {
  symbol: string
  label: string
}

interface IndexRow {
  symbol: string
  label: string
  quote: Quote | undefined
  trend: 'up' | 'down' | 'flat'
}

const store = useMarketStore()
const defaultSymbols = ['AAPL', 'TSLA', 'MSFT', 'GOOGL', 'AMZN', 'NVDA', 'INTC', 'QCOM']

const indexDefs: IndexDef[] = [
  { symbol: '^GSPC', label: '标普500' },
  { symbol: '^DJI', label: '道琼斯' },
  { symbol: 'QQQ', label: '纳斯达克100' },
  { symbol: '^VIX', label: '恐慌指数' },
]
const now = ref(new Date())
const loadingSymbols = ref<Set<string>>(new Set())
let clockTimer: number | undefined

const quoteRows = computed<QuoteRow[]>(() =>
  defaultSymbols.map((symbol) => ({
    symbol,
    quote: store.quotes.get(symbol),
    loading: loadingSymbols.value.has(symbol),
  })),
)

const indexRows = computed<IndexRow[]>(() =>
  indexDefs.map((def) => {
    const quote = store.quotes.get(def.symbol)
    let trend: 'up' | 'down' | 'flat' = 'flat'
    if (quote && quote.change > 0) trend = 'up'
    else if (quote && quote.change < 0) trend = 'down'
    return { symbol: def.symbol, label: def.label, quote, trend }
  }),
)

const isRefreshing = computed(() => loadingSymbols.value.size > 0)
const chinaClock = computed(() => formatClock(now.value, 'Asia/Shanghai'))
const usClock = computed(() => formatClock(now.value, 'America/New_York'))

function formatVolume(v: number): string {
  if (v >= 1e9) return (v / 1e9).toFixed(1) + 'B'
  if (v >= 1e6) return (v / 1e6).toFixed(1) + 'M'
  if (v >= 1e3) return (v / 1e3).toFixed(0) + 'K'
  return v.toString()
}

function formatClock(date: Date, timeZone: string): ClockParts {
  return {
    time: new Intl.DateTimeFormat('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
      timeZone,
    }).format(date),
    date: new Intl.DateTimeFormat('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      weekday: 'short',
      timeZone,
    }).format(date),
  }
}

function formatQuoteTime(timestamp?: number): string {
  if (!timestamp) return '暂无缓存'
  return `更新 ${new Date(timestamp * 1000).toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })}`
}

function setSymbolLoading(symbol: string, loading: boolean) {
  const next = new Set(loadingSymbols.value)
  if (loading) next.add(symbol)
  else next.delete(symbol)
  loadingSymbols.value = next
}

async function refreshQuote(symbol: string) {
  setSymbolLoading(symbol, true)
  try {
    await store.fetchQuote(symbol)
  } catch (error) {
    console.warn(`quote refresh failed: ${symbol}`, error)
  } finally {
    setSymbolLoading(symbol, false)
  }
}

async function fetchIndices() {
  const symbols = indexDefs.map((d) => d.symbol)
  try {
    await store.fetchQuotes(symbols)
  } catch {
    await Promise.allSettled(symbols.map((s) => store.fetchQuote(s)))
  }
}

async function refreshQuotes() {
  if (loadingSymbols.value.size) return
  loadingSymbols.value = new Set(defaultSymbols)
  try {
    const quotes = await store.fetchQuotes(defaultSymbols)
    const fetchedSymbols = new Set(quotes.map((q) => q.symbol.toUpperCase()))
    const missing = defaultSymbols.filter((s) => !fetchedSymbols.has(s))
    if (missing.length) {
      await Promise.allSettled(missing.map((s) => refreshQuote(s)))
    }
  } catch (error) {
    console.warn('batch quote refresh failed', error)
    await Promise.allSettled(defaultSymbols.map((s) => refreshQuote(s)))
  } finally {
    loadingSymbols.value = new Set()
  }
}

async function refreshAll() {
  await Promise.all([fetchIndices(), refreshQuotes()])
}

onMounted(async () => {
  clockTimer = window.setInterval(() => { now.value = new Date() }, 1000)
  void refreshAll()
})

onBeforeUnmount(() => {
  if (clockTimer) window.clearInterval(clockTimer)
})
</script>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 100%;
}

.dashboard-overview {
  display: grid;
  grid-template-columns: minmax(220px, 0.7fr) minmax(0, 1.3fr);
  gap: 18px;
  align-items: stretch;
  padding: 18px;
}

.dashboard-status,
.clock-card,
.index-card {
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.dashboard-status { gap: 8px; }

.status-label,
.clock-card span,
.clock-card small,
.quote-updated {
  color: var(--neu-text-dim);
  font-size: 13px;
}

.dashboard-status strong {
  color: var(--neu-primary);
  font-size: 24px;
  font-weight: 800;
}

.refresh-button {
  width: fit-content;
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 9px 14px;
  background: var(--neu-bg);
  color: var(--neu-text);
  cursor: pointer;
  font-weight: 800;
  box-shadow:
    3px 3px 6px var(--neu-shadow-dark),
    -3px -3px 6px var(--neu-shadow-light);
}

.refresh-button:disabled {
  cursor: progress;
  opacity: 0.62;
}

.clock-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.clock-card {
  min-height: 112px;
  padding: 16px 18px;
}

.clock-card strong {
  margin: 6px 0;
  color: var(--neu-text);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0;
}

/* Index strip */
.dashboard-indices {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}

.index-card {
  min-height: 110px;
  padding: 16px 18px;
  gap: 4px;
}

.index-name {
  color: var(--neu-text-dim);
  font-size: 13px;
  font-weight: 600;
}

.index-price {
  margin: 2px 0;
  font-size: 24px;
  font-weight: 800;
  letter-spacing: 0;
  color: var(--neu-text);
}

.index-price.up { color: var(--neu-success); }
.index-price.down { color: var(--neu-danger); }

.index-change {
  font-size: 14px;
  font-weight: 600;
  color: var(--neu-text-dim);
}

.index-change.up { color: var(--neu-success); }
.index-change.down { color: var(--neu-danger); }

.index-change small {
  margin-left: 6px;
  font-size: 12px;
  opacity: 0.85;
}

.muted {
  color: var(--neu-text-dim) !important;
}

/* Quote cards */
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
}

.quote-card {
  cursor: default;
  min-height: 154px;
}

.quote-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 8px;
}

.quote-symbol {
  font-size: 18px;
  font-weight: 700;
  color: var(--neu-primary);
}

.quote-price {
  font-size: 22px;
  font-weight: 700;
}

.quote-change {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 12px;
}

.quote-change.up { color: var(--neu-success); }
.quote-change.down { color: var(--neu-danger); }

.quote-percent {
  font-size: 13px;
  opacity: 0.8;
}

.quote-volume {
  font-size: 13px;
  color: var(--neu-text-dim);
  margin-bottom: 8px;
}

.quote-updated {
  min-height: 18px;
}

.quote-updated.refreshing {
  color: var(--neu-primary);
}

@media (max-width: 920px) {
  .dashboard-indices {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 820px) {
  .dashboard-overview,
  .clock-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 560px) {
  .dashboard-indices {
    grid-template-columns: 1fr;
  }
}
</style>
