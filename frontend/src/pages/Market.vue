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

        <div class="quote-actions">
          <button
            class="plain-action"
            type="button"
            @click="isTracked ? removeActiveFromWatchlist() : addActiveToWatchlist()"
          >
            {{ isTracked ? '移出自选' : '加入自选' }}
          </button>
          <button class="plain-action" type="button" @click="exportActiveHistory">导出CSV</button>
          <button class="plain-action danger" type="button" @click="clearActiveHistory">清空时间线</button>
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
          <div class="quote-detail-row">
            <span>提醒</span>
            <strong>{{ activeAlerts.length }} 个</strong>
          </div>
          <div class="quote-detail-row">
            <span>历史</span>
            <strong>{{ historyStatus }}</strong>
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

        <EChartsLine
          :points="chartPoints"
          :accent="trendAccent"
          :markers="filteredMarkers"
          :selected-time="selectedMarker?.time"
          :baseline-price="timelineStats?.open"
          @marker-select="selectMarker"
        />

        <div class="chart-footer">
          <span>{{ chartPoints.length }} 点 · {{ filteredMarkers.length }} 个事件</span>
          <span v-if="errorMessage" class="chart-error">{{ errorMessage }}</span>
          <span v-else-if="historyError" class="chart-error">{{ historyError }}</span>
          <span v-else>{{ isLoading || isHistoryLoading ? '更新中' : '已同步' }}</span>
        </div>

        <div v-if="timelineStats" class="timeline-stats">
          <div class="stat-tile">
            <span>区间涨跌</span>
            <strong :class="timelineStats.change >= 0 ? 'up' : 'down'">
              {{ signedPrice(timelineStats.change) }}
            </strong>
            <small>{{ signedPercent(timelineStats.changePercent) }}</small>
          </div>
          <div class="stat-tile">
            <span>最高 / 最低</span>
            <strong>{{ formatPrice(timelineStats.high.price) }}</strong>
            <small>{{ formatPrice(timelineStats.low.price) }}</small>
          </div>
          <div class="stat-tile">
            <span>振幅</span>
            <strong>{{ timelineStats.amplitude.toFixed(2) }}%</strong>
            <small>{{ timelineStats.points }} 点</small>
          </div>
          <div class="stat-tile">
            <span>均量</span>
            <strong>{{ formatVolume(timelineStats.avgVolume) }}</strong>
            <small>{{ formatMarketTime(timelineStats.last.time) }}</small>
          </div>
        </div>
      </NeuCard>
    </section>

    <section class="timeline-workbench">
      <NeuCard class="timeline-panel">
        <div class="panel-head">
          <div>
            <h3>时间线</h3>
            <p>{{ markerSummary }}</p>
          </div>
          <div class="marker-tabs">
            <button
              v-for="option in markerFilters"
              :key="option.key"
              class="marker-tab"
              :class="{ 'marker-tab--active': option.key === activeMarkerKind }"
              type="button"
              @click="activeMarkerKind = option.key"
            >
              {{ option.label }}
            </button>
          </div>
        </div>

        <div v-if="selectedMarker" class="selected-event neu-concave">
          <span :class="['event-dot', `event-dot--${selectedMarker.tone}`]"></span>
          <div>
            <strong>{{ selectedMarker.title }}</strong>
            <p>{{ selectedMarker.detail }}</p>
            <small>{{ formatMarkerTime(selectedMarker.time) }} · {{ formatPrice(selectedMarker.price) }}</small>
          </div>
        </div>

        <div v-if="filteredMarkers.length" class="event-list">
          <button
            v-for="marker in filteredMarkers"
            :key="marker.id"
            class="event-row"
            :class="{ 'event-row--active': selectedMarker?.id === marker.id }"
            type="button"
            @click="selectMarker(marker)"
          >
            <span :class="['event-dot', `event-dot--${marker.tone}`]"></span>
            <span class="event-main">
              <strong>{{ marker.title }}</strong>
              <small>{{ marker.detail }}</small>
            </span>
            <span class="event-meta">
              {{ formatMarketTime(marker.time) }}
              <small>{{ formatPrice(marker.price) }}</small>
            </span>
          </button>
        </div>
        <div v-else class="empty-state">当前窗口还没有可显示的事件</div>
      </NeuCard>

      <NeuCard class="tools-panel">
        <div class="tool-block">
          <div class="panel-head compact">
            <div>
              <h3>笔记</h3>
              <p>{{ activeNotes.length }} 条</p>
            </div>
          </div>
          <div class="note-form">
            <textarea v-model="noteText" class="neu-textarea" maxlength="120" placeholder="记录此刻价格、原因或观察..."></textarea>
            <button class="plain-action primary" type="button" :disabled="!canAddNote" @click="addTimelineNote">
              标记到时间线
            </button>
          </div>
          <div v-if="activeNotes.length" class="note-list">
            <div v-for="note in activeNotes" :key="note.id" class="note-row neu-concave">
              <div>
                <strong>{{ formatMarkerTime(note.time) }}</strong>
                <p>{{ note.text }}</p>
              </div>
              <button class="mini-action" type="button" @click="store.removeTimelineNote(activeSymbol, note.id)">删除</button>
            </div>
          </div>
        </div>

        <div class="tool-block">
          <div class="panel-head compact">
            <div>
              <h3>价格提醒</h3>
              <p>{{ triggeredAlertCount }} 个已触发</p>
            </div>
          </div>
          <div class="alert-form">
            <div class="direction-tabs">
              <button
                class="direction-tab"
                :class="{ 'direction-tab--active': alertDirection === 'above' }"
                type="button"
                @click="alertDirection = 'above'"
              >
                突破
              </button>
              <button
                class="direction-tab"
                :class="{ 'direction-tab--active': alertDirection === 'below' }"
                type="button"
                @click="alertDirection = 'below'"
              >
                跌破
              </button>
            </div>
            <input v-model.number="alertTarget" class="neu-input alert-input" type="number" min="0" step="0.01" />
            <button class="plain-action primary" type="button" :disabled="!canAddAlert" @click="addPriceAlert">添加</button>
          </div>
          <div v-if="activeAlerts.length" class="alert-list">
            <div
              v-for="alert in activeAlerts"
              :key="alert.id"
              class="alert-row neu-concave"
              :class="{ 'alert-row--triggered': alert.triggered_at }"
            >
              <div>
                <strong>{{ alert.direction === 'above' ? '突破' : '跌破' }} {{ formatPrice(alert.target) }}</strong>
                <p>{{ alert.triggered_at ? `触发于 ${formatMarkerTime(alert.triggered_at)}` : '等待触发' }}</p>
              </div>
              <div class="alert-actions">
                <button class="mini-action" type="button" @click="store.togglePriceAlert(alert.id)">
                  {{ alert.enabled ? '暂停' : '启用' }}
                </button>
                <button v-if="alert.triggered_at" class="mini-action" type="button" @click="store.resetPriceAlert(alert.id)">重置</button>
                <button class="mini-action danger" type="button" @click="store.removePriceAlert(alert.id)">删除</button>
              </div>
            </div>
          </div>
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
import EChartsLine from '../components/EChartsLine.vue'
import { useMarketStore } from '../stores/market'
import type { PriceAlert, QuotePoint, TimelineMarker, TimelineMarkerKind, TimelineNote } from '../types'

type FrameKey = 'second' | 'minute' | 'hour' | 'day' | 'all'
type MarkerFilterKey = TimelineMarkerKind | 'all'

interface FrameOption {
  key: FrameKey
  shortLabel: string
  label: string
  pollMs: number
  bucketMs: number
  windowMs: number
}

interface TimelineStats {
  open: number
  last: QuotePoint
  high: QuotePoint
  low: QuotePoint
  change: number
  changePercent: number
  amplitude: number
  avgVolume: number
  points: number
}

interface HistoryRequest {
  interval: string
  range: string
}

const frames: FrameOption[] = [
  { key: 'second', shortLabel: '秒', label: '秒', pollMs: 1000, bucketMs: 1000, windowMs: 5 * 60 * 1000 },
  { key: 'minute', shortLabel: '分', label: '分钟', pollMs: 5000, bucketMs: 60 * 1000, windowMs: 2 * 60 * 60 * 1000 },
  { key: 'hour', shortLabel: '时', label: '小时', pollMs: 30 * 1000, bucketMs: 60 * 60 * 1000, windowMs: 24 * 60 * 60 * 1000 },
  { key: 'day', shortLabel: '天', label: '日', pollMs: 60 * 1000, bucketMs: 24 * 60 * 60 * 1000, windowMs: 30 * 24 * 60 * 60 * 1000 },
  { key: 'all', shortLabel: '全', label: '全部', pollMs: 60 * 1000, bucketMs: 60 * 60 * 1000, windowMs: Number.POSITIVE_INFINITY },
]

const markerFilters: Array<{ key: MarkerFilterKey; label: string }> = [
  { key: 'all', label: '全部' },
  { key: 'high', label: '高低' },
  { key: 'move', label: '异动' },
  { key: 'session', label: '时段' },
  { key: 'note', label: '笔记' },
  { key: 'alert', label: '提醒' },
]

const MARKET_TIME_ZONE = 'America/New_York'
const store = useMarketStore()
const query = ref('AAPL')
const activeSymbol = ref('AAPL')
const activeFrameKey = ref<FrameKey>('second')
const activeMarkerKind = ref<MarkerFilterKey>('all')
const autoRefresh = ref(true)
const isLoading = ref(false)
const isHistoryLoading = ref(false)
const errorMessage = ref('')
const historyError = ref('')
const noteText = ref('')
const alertDirection = ref<PriceAlert['direction']>('above')
const alertTarget = ref<number>()
const selectedMarker = ref<TimelineMarker>()
let timerId: number | undefined
let refreshInFlight = false

const activeFrame = computed(() =>
  frames.find((frame) => frame.key === activeFrameKey.value) ?? frames[0],
)
const historyRequest = computed(() => historyRequestForFrame(activeFrameKey.value))
const quickSymbols = computed(() => store.trackedSymbols)
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
const isTracked = computed(() => quickSymbols.value.includes(activeSymbol.value))
const activeNotes = computed(() => store.notesFor(activeSymbol.value))
const activeAlerts = computed(() =>
  store.priceAlerts.filter((alert) => alert.symbol === activeSymbol.value),
)
const triggeredAlertCount = computed(() =>
  activeAlerts.value.filter((alert) => alert.triggered_at).length,
)
const canAddNote = computed(() =>
  Boolean(noteText.value.trim()) && Boolean(latestPoint.value),
)
const canAddAlert = computed(() =>
  Number.isFinite(alertTarget.value) && Number(alertTarget.value) > 0,
)
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
  const cutoff = Number.isFinite(activeFrame.value.windowMs)
    ? Date.now() - activeFrame.value.windowMs
    : 0
  return store
    .historyFor(activeSymbol.value)
    .filter((point) => point.time >= cutoff)
    .sort((a, b) => a.time - b.time)
})
const chartPoints = computed(() => aggregatePoints(rawPoints.value, activeFrame.value.bucketMs))
const latestPoint = computed(() => chartPoints.value[chartPoints.value.length - 1])
const timelineStats = computed<TimelineStats | undefined>(() => {
  const points = chartPoints.value
  if (!points.length) return undefined

  const first = points[0]
  const last = points[points.length - 1]
  const high = points.reduce((best, point) => point.price > best.price ? point : best, first)
  const low = points.reduce((best, point) => point.price < best.price ? point : best, first)
  const change = last.price - first.price
  const changePercent = first.price === 0 ? 0 : (change / first.price) * 100
  const amplitude = first.price === 0 ? 0 : ((high.price - low.price) / first.price) * 100
  const avgVolume = points.reduce((sum, point) => sum + (point.volume || 0), 0) / points.length

  return {
    open: first.price,
    last,
    high,
    low,
    change,
    changePercent,
    amplitude,
    avgVolume,
    points: points.length,
  }
})
const timelineMarkers = computed<TimelineMarker[]>(() => {
  const points = chartPoints.value
  const notes = activeNotes.value.map(noteToMarker).filter(markerInFrame)
  const alerts = activeAlerts.value
    .filter((alert) => alert.triggered_at)
    .map(alertToMarker)
    .filter(markerInFrame)
  if (points.length < 2) return [...notes, ...alerts].sort((a, b) => b.time - a.time)

  return [
    ...buildPriceMarkers(points),
    ...buildSessionMarkers(points),
    ...notes,
    ...alerts,
  ].sort((a, b) => b.time - a.time)
})
const filteredMarkers = computed(() => {
  if (activeMarkerKind.value === 'all') return timelineMarkers.value
  if (activeMarkerKind.value === 'high') {
    return timelineMarkers.value.filter((marker) => marker.kind === 'high' || marker.kind === 'low')
  }
  return timelineMarkers.value.filter((marker) => marker.kind === activeMarkerKind.value)
})
const markerSummary = computed(() =>
  `${frameWindowLabel.value} · ${timelineMarkers.value.length} 个时间点标记`,
)
const historyStatus = computed(() => {
  if (isHistoryLoading.value) return '加载中'
  if (historyError.value) return '失败'
  if (!chartPoints.value.length) return '未加载'
  return `${historyRequest.value.interval}/${historyRequest.value.range}`
})
const lastUpdated = computed(() => {
  if (!currentQuote.value) return '未同步'
  const time = new Intl.DateTimeFormat('zh-CN', {
    timeZone: MARKET_TIME_ZONE,
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(currentQuote.value.timestamp * 1000))

  return `美东 ${time}`
})
const frameWindowLabel = computed(() => {
  if (activeFrameKey.value === 'second') return '近 5 分钟'
  if (activeFrameKey.value === 'minute') return '近 2 小时'
  if (activeFrameKey.value === 'hour') return '近 24 小时'
  if (activeFrameKey.value === 'day') return '近 30 天'
  return '全部缓存'
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

function historyRequestForFrame(frame: FrameKey): HistoryRequest {
  if (frame === 'second') return { interval: '1m', range: '1d' }
  if (frame === 'minute') return { interval: '1m', range: '1d' }
  if (frame === 'hour') return { interval: '5m', range: '5d' }
  if (frame === 'day') return { interval: '1d', range: '1mo' }
  return { interval: '1d', range: '6mo' }
}

function buildPriceMarkers(points: QuotePoint[]): TimelineMarker[] {
  const first = points[0]
  const high = points.reduce((best, point) => point.price > best.price ? point : best, first)
  const low = points.reduce((best, point) => point.price < best.price ? point : best, first)
  const moves = points
    .slice(1)
    .map((point, index) => {
      const prev = points[index]
      const pct = prev.price === 0 ? 0 : ((point.price - prev.price) / prev.price) * 100
      return { point, pct }
    })
    .filter((item) => Math.abs(item.pct) >= moveThreshold())
    .sort((a, b) => Math.abs(b.pct) - Math.abs(a.pct))
    .slice(0, 6)
    .map(({ point, pct }) => ({
      id: `move-${point.time}-${pct.toFixed(2)}`,
      symbol: activeSymbol.value,
      time: point.time,
      price: point.price,
      kind: 'move' as TimelineMarkerKind,
      title: pct >= 0 ? '快速拉升' : '快速回落',
      detail: `${formatMarketTime(point.time)} 单桶 ${signedPercent(pct)}`,
      tone: pct >= 0 ? 'up' as const : 'down' as const,
    }))
  const volumeSpikes = volumeSpikeMarkers(points)

  return [
    {
      id: `high-${high.time}`,
      symbol: activeSymbol.value,
      time: high.time,
      price: high.price,
      kind: 'high',
      title: '区间最高',
      detail: `${formatMarketTime(high.time)} 达到 ${formatPrice(high.price)}`,
      tone: 'up',
    },
    {
      id: `low-${low.time}`,
      symbol: activeSymbol.value,
      time: low.time,
      price: low.price,
      kind: 'low',
      title: '区间最低',
      detail: `${formatMarketTime(low.time)} 到达 ${formatPrice(low.price)}`,
      tone: 'down',
    },
    ...moves,
    ...volumeSpikes,
  ]
}

function buildSessionMarkers(points: QuotePoint[]): TimelineMarker[] {
  const markers: TimelineMarker[] = []
  for (let i = 1; i < points.length; i++) {
    const prev = points[i - 1]
    const point = points[i]
    if (!point.session || !prev.session || point.session === prev.session) continue
    markers.push({
      id: `session-${point.time}-${point.session}`,
      symbol: activeSymbol.value,
      time: point.time,
      price: point.price,
      kind: 'session',
      title: `进入${sessionLabelFor(point.session)}`,
      detail: `${formatMarketTime(point.time)} 从${sessionLabelFor(prev.session)}切换`,
      tone: 'info',
    })
  }
  return markers.slice(-8)
}

function volumeSpikeMarkers(points: QuotePoint[]): TimelineMarker[] {
  const volumePoints = points.filter((point) => point.volume > 0)
  if (volumePoints.length < 6) return []

  const avg = volumePoints.reduce((sum, point) => sum + point.volume, 0) / volumePoints.length
  return volumePoints
    .filter((point) => point.volume > avg * 2.2)
    .sort((a, b) => b.volume - a.volume)
    .slice(0, 3)
    .map((point) => ({
      id: `volume-${point.time}`,
      symbol: activeSymbol.value,
      time: point.time,
      price: point.price,
      kind: 'volume' as TimelineMarkerKind,
      title: '成交放量',
      detail: `${formatVolume(point.volume)}，约为均量 ${(point.volume / avg).toFixed(1)} 倍`,
      tone: 'warning' as const,
    }))
}

function noteToMarker(note: TimelineNote): TimelineMarker {
  return {
    id: note.id,
    symbol: note.symbol,
    time: note.time,
    price: note.price,
    kind: 'note',
    title: '手动笔记',
    detail: note.text,
    tone: 'info',
  }
}

function alertToMarker(alert: PriceAlert): TimelineMarker {
  const time = alert.triggered_at ?? alert.created_at
  const point = nearestPoint(time)
  return {
    id: alert.id,
    symbol: alert.symbol,
    time,
    price: point?.price ?? alert.target,
    kind: 'alert',
    title: alert.direction === 'above' ? '突破提醒触发' : '跌破提醒触发',
    detail: `${alert.direction === 'above' ? '突破' : '跌破'} ${formatPrice(alert.target)}`,
    tone: alert.direction === 'above' ? 'up' : 'down',
  }
}

function nearestPoint(time: number): QuotePoint | undefined {
  return store.historyFor(activeSymbol.value).reduce<QuotePoint | undefined>((best, point) => {
    if (!best) return point
    return Math.abs(point.time - time) < Math.abs(best.time - time) ? point : best
  }, undefined)
}

function markerInFrame(marker: TimelineMarker): boolean {
  if (!Number.isFinite(activeFrame.value.windowMs)) return true
  return marker.time >= Date.now() - activeFrame.value.windowMs
}

function moveThreshold(): number {
  if (activeFrameKey.value === 'second') return 0.15
  if (activeFrameKey.value === 'minute') return 0.35
  if (activeFrameKey.value === 'hour') return 0.75
  if (activeFrameKey.value === 'day') return 1.5
  return 2
}

function formatVolume(value: number): string {
  if (value >= 1e9) return `${(value / 1e9).toFixed(1)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(1)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(0)}K`
  return Math.round(value).toString()
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

function formatOptionalPrice(value: number | undefined): string {
  return isFiniteNumber(value) ? formatPrice(value) : '--'
}

function formatPrice(value: number): string {
  return `$${value.toFixed(2)}`
}

function signedPrice(value: number): string {
  return `${value >= 0 ? '+' : ''}${formatPrice(value)}`
}

function signedPercent(value: number): string {
  return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`
}

function formatMarketTime(time: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    timeZone: MARKET_TIME_ZONE,
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: activeFrameKey.value === 'second' ? '2-digit' : undefined,
  }).format(new Date(time))
}

function formatMarkerTime(time: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    timeZone: MARKET_TIME_ZONE,
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(time))
}

function sessionLabelFor(session?: string): string {
  if (session === 'pre_market') return '盘前'
  if (session === 'post_market') return '盘后'
  return '常规盘'
}

function selectMarker(marker: TimelineMarker) {
  selectedMarker.value = marker
}

function addActiveToWatchlist() {
  store.addTrackedSymbol(activeSymbol.value)
}

function removeActiveFromWatchlist() {
  store.removeTrackedSymbol(activeSymbol.value)
}

function addTimelineNote() {
  const point = latestPoint.value
  if (!point) return
  const note = store.addTimelineNote(activeSymbol.value, point.time, point.price, noteText.value)
  if (!note) return

  noteText.value = ''
  selectedMarker.value = noteToMarker(note)
  activeMarkerKind.value = 'note'
}

function addPriceAlert() {
  const target = Number(alertTarget.value)
  if (!Number.isFinite(target) || target <= 0) return

  const alert = store.addPriceAlert(activeSymbol.value, alertDirection.value, target)
  if (alert) activeMarkerKind.value = 'alert'
}

function exportActiveHistory() {
  const rows = store.historyFor(activeSymbol.value)
  if (!rows.length) return

  const csv = [
    'symbol,time_iso,price,change,change_percent,volume,session',
    ...rows.map((point) => [
      point.symbol,
      new Date(point.time).toISOString(),
      point.price,
      point.change,
      point.change_percent,
      point.volume,
      point.session ?? '',
    ].join(',')),
  ].join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${activeSymbol.value}-timeline.csv`
  link.click()
  URL.revokeObjectURL(url)
}

function clearActiveHistory() {
  if (!window.confirm(`清空 ${activeSymbol.value} 的本地时间线缓存？`)) return
  store.clearHistory(activeSymbol.value)
  selectedMarker.value = undefined
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

async function loadHistory() {
  isHistoryLoading.value = true
  historyError.value = ''

  try {
    await store.fetchHistory(activeSymbol.value, {
      ...historyRequest.value,
      extendedHours: true,
    })
  } catch (error) {
    historyError.value = error instanceof Error ? error.message : '历史加载失败'
  } finally {
    isHistoryLoading.value = false
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
  selectedMarker.value = undefined
  if (currentQuote.value) alertTarget.value = Number(currentQuote.value.price.toFixed(2))
  await Promise.all([loadHistory(), refreshQuote()])
  if (currentQuote.value) alertTarget.value = Number(currentQuote.value.price.toFixed(2))
  restartTimer()
}

watch(autoRefresh, restartTimer)
watch(activeFrameKey, async () => {
  selectedMarker.value = undefined
  restartTimer()
  await loadHistory()
})
watch(currentQuote, (quote) => {
  if (quote && !alertTarget.value) alertTarget.value = Number(quote.price.toFixed(2))
})

onMounted(async () => {
  await Promise.all([loadHistory(), refreshQuote()])
  if (currentQuote.value) alertTarget.value = Number(currentQuote.value.price.toFixed(2))
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

.neu-input,
.neu-textarea {
  width: 100%;
  min-width: 0;
  border: none;
  border-radius: var(--neu-radius-sm);
  background: var(--neu-bg);
  box-shadow:
    inset 3px 3px 6px var(--neu-shadow-dark),
    inset -3px -3px 6px var(--neu-shadow-light);
  color: var(--neu-text);
  outline: none;
  transition: box-shadow var(--neu-transition);
}

.neu-input {
  flex: 1;
  padding: 14px 20px;
  font-size: 16px;
}

.neu-textarea {
  min-height: 76px;
  resize: vertical;
  padding: 12px 14px;
  font: inherit;
}

.neu-input:focus,
.neu-textarea:focus {
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
.frame-tab,
.marker-tab,
.direction-tab,
.plain-action,
.mini-action {
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
.frame-tab--active,
.marker-tab--active,
.direction-tab--active {
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
.chart-footer,
.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.panel-head.compact {
  margin-bottom: 12px;
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
.chart-footer,
.panel-head p,
.event-row small,
.selected-event p,
.selected-event small,
.note-row p,
.alert-row p,
.empty-state,
.stat-tile span,
.stat-tile small {
  color: var(--neu-text-dim);
  font-size: 13px;
}

.panel-head h3,
.panel-head p {
  margin: 0;
}

.panel-head h3 {
  font-size: 18px;
  font-weight: 800;
}

.panel-head p {
  margin-top: 4px;
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

.quote-actions,
.marker-tabs,
.direction-tabs,
.alert-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.plain-action {
  padding: 9px 12px;
  font-weight: 800;
}

.plain-action.primary {
  color: #fff;
  background: var(--neu-primary);
}

.plain-action.danger,
.mini-action.danger {
  color: var(--neu-danger);
}

.plain-action:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.mini-action {
  padding: 6px 9px;
  font-size: 12px;
  font-weight: 800;
}

.quote-price.up,
.quote-change.up,
.mr-change.up,
.stat-tile strong.up {
  color: var(--neu-success);
}

.quote-price.down,
.quote-change.down,
.mr-change.down,
.stat-tile strong.down {
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

.frame-tabs,
.marker-tabs,
.direction-tabs {
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

.marker-tab,
.direction-tab {
  min-height: 32px;
  padding: 0 11px;
  font-weight: 800;
}

.chart-footer {
  min-height: 18px;
  margin-top: 12px;
}

.chart-error {
  color: var(--neu-danger);
}

.timeline-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
  margin-top: 14px;
}

.stat-tile {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
  border-radius: var(--neu-radius-sm);
  padding: 12px;
  background: rgba(255, 255, 255, 0.34);
}

.stat-tile strong {
  font-size: 18px;
}

.timeline-workbench {
  display: grid;
  grid-template-columns: minmax(0, 1.3fr) minmax(300px, 0.7fr);
  gap: 24px;
}

.selected-event {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  align-items: start;
  margin-top: 16px;
  padding: 14px;
}

.selected-event strong,
.note-row strong,
.alert-row strong {
  color: var(--neu-text);
}

.selected-event p,
.note-row p,
.alert-row p {
  margin: 4px 0 0;
}

.event-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 360px;
  margin-top: 16px;
  overflow: auto;
  padding-right: 4px;
}

.event-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  width: 100%;
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 12px;
  color: var(--neu-text);
  background: rgba(255, 255, 255, 0.28);
  text-align: left;
  cursor: pointer;
}

.event-row--active {
  box-shadow:
    inset 2px 2px 5px var(--neu-shadow-dark),
    inset -2px -2px 5px var(--neu-shadow-light);
}

.event-main,
.event-meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 3px;
}

.event-main strong,
.event-main small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.event-meta {
  align-items: flex-end;
  color: var(--neu-text-dim);
  font-size: 12px;
}

.event-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--neu-text-dim);
  box-shadow: 0 0 0 4px rgba(142, 142, 154, 0.12);
}

.event-dot--up {
  background: var(--neu-success);
  box-shadow: 0 0 0 4px rgba(76, 217, 100, 0.14);
}

.event-dot--down {
  background: var(--neu-danger);
  box-shadow: 0 0 0 4px rgba(255, 91, 91, 0.14);
}

.event-dot--info {
  background: var(--neu-primary);
  box-shadow: 0 0 0 4px rgba(91, 127, 255, 0.14);
}

.event-dot--warning {
  background: #f5a623;
  box-shadow: 0 0 0 4px rgba(245, 166, 35, 0.18);
}

.tools-panel,
.tool-block,
.note-form,
.note-list,
.alert-form,
.alert-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tool-block + .tool-block {
  border-top: 1px solid rgba(142, 142, 154, 0.18);
  padding-top: 18px;
}

.alert-form {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
}

.alert-input {
  padding: 10px 12px;
  font-size: 14px;
}

.note-row,
.alert-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  padding: 12px;
}

.alert-row--triggered {
  outline: 1px solid rgba(91, 127, 255, 0.25);
}

.empty-state {
  display: grid;
  min-height: 120px;
  place-items: center;
  text-align: center;
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

@media (max-width: 1080px) {
  .timeline-workbench,
  .market-overview {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  .market-toolbar {
    grid-template-columns: 1fr;
  }

  .symbol-strip {
    justify-content: flex-start;
  }

  .timeline-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 560px) {
  .market-search,
  .chart-panel__head,
  .panel-head {
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

  .timeline-stats,
  .alert-form {
    grid-template-columns: 1fr;
  }

  .event-row {
    grid-template-columns: auto minmax(0, 1fr);
  }

  .event-meta {
    grid-column: 2;
    align-items: flex-start;
  }
}
</style>
