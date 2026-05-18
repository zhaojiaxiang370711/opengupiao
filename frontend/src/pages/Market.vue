<template>
  <div class="market">
    <section class="market-toolbar neu-convex">
      <div class="market-search">
        <input
          v-model="query"
          class="neu-input"
          placeholder="搜索美股代码..."
          @keyup.enter="handleSearch"
        />
        <NeuButton variant="primary" @click="handleSearch">搜索</NeuButton>
      </div>

      <div class="symbol-strip">
        <button
          v-for="symbol in quickSymbols"
          :key="symbol"
          class="symbol-chip"
          :class="{ 'symbol-chip--active': symbol === activeSymbol }"
          type="button"
          @click="selectSymbol(symbol)"
        >
          {{ symbol }}
        </button>
      </div>
    </section>

    <section class="market-overview">
      <NeuCard class="quote-panel">
        <div class="quote-panel__head">
          <div>
            <div class="quote-title">
              <div class="quote-symbol">{{ activeSymbol }}</div>
              <span
                v-if="currentQuote"
                class="session-badge"
                :class="`session-badge--${sessionClass}`"
              >
                {{ sessionLabel }}
              </span>
            </div>
            <div class="quote-time">{{ lastUpdated }}</div>
          </div>
          <label class="live-toggle">
            <input v-model="autoRefresh" type="checkbox" />
            <span>实时</span>
          </label>
        </div>

        <div class="quote-price" :class="trendClass">
          {{ currentQuote ? `$${currentQuote.price.toFixed(2)}` : '--' }}
        </div>
        <div v-if="currentQuote" class="quote-change" :class="trendClass">
          {{ currentQuote.change >= 0 ? '+' : '' }}{{ currentQuote.change.toFixed(2) }}
          <span>{{ currentQuote.change_percent >= 0 ? '+' : '' }}{{ currentQuote.change_percent.toFixed(2) }}%</span>
        </div>
        <div class="quote-details">
          <div class="quote-detail-row">
            <span>成交量</span>
            <strong>{{ currentQuote ? formatVolume(currentQuote.volume) : '--' }}</strong>
          </div>
          <div
            v-for="row in sessionRows"
            :key="row.key"
            class="quote-detail-row"
          >
            <span>{{ row.label }}</span>
            <strong>
              {{ formatOptionalPrice(row.price) }}
              <small v-if="isFiniteNumber(row.change_percent)" :class="row.change_percent >= 0 ? 'up' : 'down'">
                {{ row.change_percent >= 0 ? '+' : '' }}{{ row.change_percent.toFixed(2) }}%
              </small>
            </strong>
          </div>
          <div v-if="currentQuote?.quote_source" class="quote-detail-row">
            <span>来源</span>
            <strong>{{ currentQuote.quote_source }}</strong>
          </div>
        </div>
      </NeuCard>

      <NeuCard class="chart-panel">
        <div class="chart-panel__head">
          <div>
            <h2>{{ activeSymbol }} 股价波动</h2>
            <p>{{ activeFrame.label }}级 · {{ frameWindowLabel }}</p>
          </div>
          <div class="frame-tabs" role="tablist">
            <button
              v-for="frame in frames"
              :key="frame.key"
              class="frame-tab"
              :class="{ 'frame-tab--active': frame.key === activeFrameKey }"
              type="button"
              @click="activeFrameKey = frame.key"
            >
              {{ frame.shortLabel }}
            </button>
          </div>
        </div>

        <PriceLineChart :points="chartPoints" :accent="trendAccent" />

        <div class="chart-footer">
          <span>{{ chartPoints.length }} 点</span>
          <span v-if="errorMessage" class="chart-error">{{ errorMessage }}</span>
          <span v-else>{{ isLoading ? '更新中' : '已同步' }}</span>
        </div>
      </NeuCard>
    </section>

    <section class="market-list" v-if="store.quotes.size">
      <div v-for="q in Array.from(store.quotes.values())" :key="q.symbol" class="market-row neu-convex">
        <span class="mr-symbol">{{ q.symbol }}</span>
        <span class="mr-price">
          ${{ q.price.toFixed(2) }}
          <small v-if="q.session && q.session !== 'regular'">{{ sessionLabelFor(q.session) }}</small>
        </span>
        <span class="mr-change" :class="q.change >= 0 ? 'up' : 'down'">
          {{ q.change >= 0 ? '+' : '' }}{{ q.change_percent.toFixed(2) }}%
        </span>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import NeuButton from '../components/NeuButton.vue'
import NeuCard from '../components/NeuCard.vue'
import PriceLineChart from '../components/PriceLineChart.vue'
import { useMarketStore } from '../stores/market'
import type { QuotePoint } from '../types'

type FrameKey = 'second' | 'minute' | 'hour' | 'day'

interface FrameOption {
  key: FrameKey
  shortLabel: string
  label: string
  pollMs: number
  bucketMs: number
  windowMs: number
}

const frames: FrameOption[] = [
  { key: 'second', shortLabel: '秒', label: '秒', pollMs: 1000, bucketMs: 1000, windowMs: 5 * 60 * 1000 },
  { key: 'minute', shortLabel: '分', label: '分钟', pollMs: 5000, bucketMs: 60 * 1000, windowMs: 2 * 60 * 60 * 1000 },
  { key: 'hour', shortLabel: '时', label: '小时', pollMs: 30 * 1000, bucketMs: 60 * 60 * 1000, windowMs: 24 * 60 * 60 * 1000 },
  { key: 'day', shortLabel: '天', label: '日', pollMs: 60 * 1000, bucketMs: 24 * 60 * 60 * 1000, windowMs: 30 * 24 * 60 * 60 * 1000 },
]

const quickSymbols = ['AAPL', 'MSFT', 'NVDA', 'TSLA', 'GOOGL', 'AMZN', 'INTC', 'QCOM']
const store = useMarketStore()
const query = ref('AAPL')
const activeSymbol = ref('AAPL')
const activeFrameKey = ref<FrameKey>('second')
const autoRefresh = ref(true)
const isLoading = ref(false)
const errorMessage = ref('')
let timerId: number | undefined
let refreshInFlight = false

const activeFrame = computed(() =>
  frames.find((frame) => frame.key === activeFrameKey.value) ?? frames[0],
)

const currentQuote = computed(() => store.quotes.get(activeSymbol.value))
const trendAccent = computed(() => {
  if (!currentQuote.value || currentQuote.value.change === 0) return 'flat'
  return currentQuote.value.change > 0 ? 'up' : 'down'
})
const trendClass = computed(() => ({
  up: trendAccent.value === 'up',
  down: trendAccent.value === 'down',
}))
const sessionLabel = computed(() => sessionLabelFor(currentQuote.value?.session))
const sessionClass = computed(() => {
  const session = currentQuote.value?.session
  if (session === 'pre_market') return 'pre'
  if (session === 'post_market') return 'post'
  return 'regular'
})
const sessionRows = computed(() => {
  const quote = currentQuote.value
  if (!quote) return []

  return [
    {
      key: 'regular',
      label: '常规盘',
      price: quote.regular_price ?? (quote.session === 'regular' ? quote.price : undefined),
      change_percent: quote.regular_change_percent,
    },
    {
      key: 'pre',
      label: '盘前',
      price: quote.pre_market_price,
      change_percent: quote.pre_market_change_percent,
    },
    {
      key: 'post',
      label: '盘后',
      price: quote.post_market_price,
      change_percent: quote.post_market_change_percent,
    },
  ].filter((row) => isFiniteNumber(row.price))
})
const rawPoints = computed(() => {
  const cutoff = Date.now() - activeFrame.value.windowMs
  return store
    .historyFor(activeSymbol.value)
    .filter((point) => point.time >= cutoff)
    .sort((a, b) => a.time - b.time)
})
const chartPoints = computed(() => aggregatePoints(rawPoints.value, activeFrame.value.bucketMs))
const lastUpdated = computed(() => {
  if (!currentQuote.value) return '未同步'
  return new Date(currentQuote.value.timestamp * 1000).toLocaleTimeString('zh-CN', {
    hour12: false,
  })
})
const frameWindowLabel = computed(() => {
  if (activeFrameKey.value === 'second') return '近 5 分钟'
  if (activeFrameKey.value === 'minute') return '近 2 小时'
  if (activeFrameKey.value === 'hour') return '近 24 小时'
  return '近 30 天'
})

function aggregatePoints(points: QuotePoint[], bucketMs: number): QuotePoint[] {
  if (points.length <= 2) return points

  const buckets = new Map<number, QuotePoint>()
  for (const point of points) {
    const bucketTime = Math.floor(point.time / bucketMs) * bucketMs
    buckets.set(bucketTime, { ...point, time: bucketTime })
  }

  const bucketed = Array.from(buckets.values()).sort((a, b) => a.time - b.time)
  return bucketed.length >= 2 ? bucketed : points.slice(-180)
}

function formatVolume(value: number): string {
  if (value >= 1e9) return `${(value / 1e9).toFixed(1)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(1)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(0)}K`
  return value.toString()
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

function formatOptionalPrice(value: number | undefined): string {
  return isFiniteNumber(value) ? `$${value.toFixed(2)}` : '--'
}

function sessionLabelFor(session?: string): string {
  if (session === 'pre_market') return '盘前'
  if (session === 'post_market') return '盘后'
  return '常规盘'
}

async function refreshQuote() {
  if (refreshInFlight) return
  refreshInFlight = true
  isLoading.value = true
  errorMessage.value = ''

  try {
    await store.fetchQuote(activeSymbol.value)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '同步失败'
  } finally {
    isLoading.value = false
    refreshInFlight = false
  }
}

function restartTimer() {
  if (timerId) {
    window.clearInterval(timerId)
    timerId = undefined
  }
  if (!autoRefresh.value) return

  timerId = window.setInterval(refreshQuote, activeFrame.value.pollMs)
}

async function handleSearch() {
  const next = query.value.trim().toUpperCase()
  if (!next) return
  await selectSymbol(next)
}

async function selectSymbol(symbol: string) {
  activeSymbol.value = symbol.toUpperCase()
  query.value = activeSymbol.value
  await refreshQuote()
  restartTimer()
}

watch([activeFrameKey, autoRefresh], restartTimer)

onMounted(async () => {
  await refreshQuote()
  restartTimer()
})

onBeforeUnmount(() => {
  if (timerId) window.clearInterval(timerId)
})
</script>

<style scoped>
.market {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 1180px;
}

.market-toolbar {
  display: grid;
  grid-template-columns: minmax(280px, 1fr) auto;
  gap: 18px;
  align-items: center;
  padding: 18px;
}

.market-search {
  display: flex;
  min-width: 0;
  gap: 12px;
}

.neu-input {
  flex: 1;
  min-width: 0;
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 14px 20px;
  font-size: 16px;
  background: var(--neu-bg);
  box-shadow:
    inset 3px 3px 6px var(--neu-shadow-dark),
    inset -3px -3px 6px var(--neu-shadow-light);
  color: var(--neu-text);
  outline: none;
  transition: box-shadow var(--neu-transition);
}

.neu-input:focus {
  box-shadow:
    inset 4px 4px 10px var(--neu-shadow-dark),
    inset -4px -4px 10px var(--neu-shadow-light);
}

.symbol-strip {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.symbol-chip,
.frame-tab {
  border: none;
  border-radius: var(--neu-radius-sm);
  color: var(--neu-text-dim);
  background: var(--neu-bg);
  cursor: pointer;
  transition: all var(--neu-transition);
}

.symbol-chip {
  min-width: 64px;
  padding: 10px 12px;
  font-weight: 700;
}

.symbol-chip--active,
.frame-tab--active {
  color: #fff;
  background: var(--neu-primary);
  box-shadow:
    3px 3px 7px var(--neu-shadow-dark),
    -3px -3px 7px var(--neu-shadow-light);
}

.market-overview {
  display: grid;
  grid-template-columns: minmax(260px, 320px) minmax(0, 1fr);
  gap: 24px;
  align-items: stretch;
}

.quote-panel {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 18px;
}

.quote-panel__head,
.chart-panel__head,
.chart-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.quote-symbol {
  color: var(--neu-primary);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0;
}

.quote-title {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.session-badge {
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  border-radius: 999px;
  padding: 0 10px;
  color: #fff;
  font-size: 12px;
  font-weight: 800;
}

.session-badge--regular {
  background: var(--neu-primary);
}

.session-badge--pre {
  background: #1f9d8a;
}

.session-badge--post {
  background: #7b61ff;
}

.quote-time,
.chart-panel__head p,
.quote-details,
.chart-footer {
  color: var(--neu-text-dim);
  font-size: 13px;
}

.live-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--neu-text);
  font-weight: 600;
}

.live-toggle input {
  width: 18px;
  height: 18px;
  accent-color: var(--neu-primary);
}

.quote-price {
  font-size: 44px;
  font-weight: 800;
  letter-spacing: 0;
}

.quote-change {
  font-size: 18px;
  font-weight: 700;
}

.quote-change span {
  margin-left: 8px;
  font-size: 15px;
}

.quote-price.up,
.quote-change.up,
.mr-change.up {
  color: var(--neu-success);
}

.quote-price.down,
.quote-change.down,
.mr-change.down {
  color: var(--neu-danger);
}

.quote-details {
  display: grid;
  gap: 10px;
  border-top: 1px solid rgba(142, 142, 154, 0.18);
  padding-top: 16px;
}

.quote-detail-row {
  display: flex;
  justify-content: space-between;
  gap: 14px;
}

.quote-detail-row strong {
  color: var(--neu-text);
  text-align: right;
}

.quote-detail-row small,
.mr-price small {
  margin-left: 6px;
  font-size: 12px;
  font-weight: 800;
}

.quote-detail-row small.up {
  color: var(--neu-success);
}

.quote-detail-row small.down {
  color: var(--neu-danger);
}

.chart-panel {
  min-width: 0;
}

.chart-panel__head {
  margin-bottom: 18px;
}

.chart-panel__head h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 800;
}

.chart-panel__head p {
  margin: 4px 0 0;
}

.frame-tabs {
  display: flex;
  flex-shrink: 0;
  gap: 8px;
  padding: 5px;
  border-radius: var(--neu-radius-sm);
  box-shadow:
    inset 2px 2px 5px var(--neu-shadow-dark),
    inset -2px -2px 5px var(--neu-shadow-light);
}

.frame-tab {
  width: 42px;
  height: 34px;
  font-weight: 800;
}

.chart-footer {
  min-height: 18px;
  margin-top: 12px;
}

.chart-error {
  color: var(--neu-danger);
}

.market-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
}

.market-row {
  display: flex;
  align-items: center;
  padding: 16px 18px;
}

.mr-symbol {
  width: 80px;
  color: var(--neu-primary);
  font-weight: 800;
}

.mr-price {
  flex: 1;
  font-size: 17px;
  font-weight: 700;
}

.mr-change {
  font-size: 15px;
  font-weight: 700;
}

@media (max-width: 900px) {
  .market-toolbar,
  .market-overview {
    grid-template-columns: 1fr;
  }

  .symbol-strip {
    justify-content: flex-start;
  }
}

@media (max-width: 560px) {
  .market-search,
  .chart-panel__head {
    flex-direction: column;
    align-items: stretch;
  }

  .quote-price {
    font-size: 36px;
  }

  .frame-tabs {
    justify-content: space-between;
  }

  .frame-tab {
    flex: 1;
  }
}
</style>
