import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { HistoryBar, PriceAlert, Quote, QuotePoint, TimelineNote } from '../types'
import { useApi } from '../composables/useApi'

const HISTORY_KEY = 'aaagupiao.quoteHistory.v1'
const TRACKED_SYMBOLS_KEY = 'aaagupiao.trackedSymbols.v1'
const TIMELINE_NOTES_KEY = 'aaagupiao.timelineNotes.v1'
const PRICE_ALERTS_KEY = 'aaagupiao.priceAlerts.v1'
const DEFAULT_SYMBOLS = ['AAPL', 'MSFT', 'NVDA', 'TSLA', 'GOOGL', 'AMZN', 'INTC', 'QCOM']
const MAX_POINTS_PER_SYMBOL = 5000
const MAX_HISTORY_AGE_MS = 30 * 24 * 60 * 60 * 1000

function readJson<T>(key: string, fallback: T): T {
  if (typeof localStorage === 'undefined') return fallback

  try {
    const raw = localStorage.getItem(key)
    return raw ? JSON.parse(raw) as T : fallback
  } catch {
    return fallback
  }
}

function writeJson(key: string, value: unknown) {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(key, JSON.stringify(value))
}

function createId(prefix: string): string {
  const random = typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 9)}`
  return `${prefix}-${random}`
}

function normalizeSymbol(symbol: string): string {
  return symbol.trim().toUpperCase()
}

function loadTrackedSymbols(): string[] {
  const stored = readJson<string[]>(TRACKED_SYMBOLS_KEY, DEFAULT_SYMBOLS)
  const merged = [...DEFAULT_SYMBOLS, ...stored]
    .map(normalizeSymbol)
    .filter(Boolean)
  return Array.from(new Set(merged)).slice(0, 24)
}

function loadHistory(): Record<string, QuotePoint[]> {
  if (typeof localStorage === 'undefined') return {}

  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw) as Record<string, QuotePoint[]>
    const cutoff = Date.now() - MAX_HISTORY_AGE_MS

    return Object.fromEntries(
      Object.entries(parsed).map(([symbol, points]) => [
        normalizeSymbol(symbol),
        points
          .filter((point) =>
            Number.isFinite(point.price)
            && point.price > 0
            && Number.isFinite(point.time)
            && point.time >= cutoff,
          )
          .slice(-MAX_POINTS_PER_SYMBOL),
      ]),
    )
  } catch {
    return {}
  }
}

function loadTimelineNotes(): Record<string, TimelineNote[]> {
  const parsed = readJson<Record<string, TimelineNote[]>>(TIMELINE_NOTES_KEY, {})
  const cutoff = Date.now() - MAX_HISTORY_AGE_MS

  return Object.fromEntries(
    Object.entries(parsed).map(([symbol, notes]) => [
      normalizeSymbol(symbol),
      notes
        .filter((note) =>
          Boolean(note.text?.trim())
          && Number.isFinite(note.time)
          && note.time >= cutoff
          && Number.isFinite(note.price),
        )
        .sort((a, b) => b.time - a.time)
        .slice(0, 120),
    ]),
  )
}

function loadPriceAlerts(): PriceAlert[] {
  return readJson<PriceAlert[]>(PRICE_ALERTS_KEY, [])
    .filter((alert) =>
      Boolean(normalizeSymbol(alert.symbol))
      && Number.isFinite(alert.target)
      && alert.target > 0
      && (alert.direction === 'above' || alert.direction === 'below'),
    )
    .map((alert) => ({
      ...alert,
      symbol: normalizeSymbol(alert.symbol),
      enabled: alert.enabled !== false,
    }))
}

function quoteFromPoint(point: QuotePoint): Quote {
  return {
    symbol: point.symbol.toUpperCase(),
    price: point.price,
    change: point.change,
    change_percent: point.change_percent,
    volume: point.volume,
    timestamp: Math.floor(point.time / 1000),
    session: point.session ?? 'regular',
  }
}

function latestQuotesFromHistory(history: Record<string, QuotePoint[]>): Map<string, Quote> {
  const next = new Map<string, Quote>()

  for (const [symbol, points] of Object.entries(history)) {
    const latest = points.reduce<QuotePoint | undefined>(
      (best, point) => (!best || point.time > best.time ? point : best),
      undefined,
    )

    if (latest) next.set(symbol.toUpperCase(), quoteFromPoint(latest))
  }

  return next
}

export const useMarketStore = defineStore('market', () => {
  const initialHistory = loadHistory()
  const quotes = ref<Map<string, Quote>>(latestQuotesFromHistory(initialHistory))
  const symbols = ref<string[]>([])
  const trackedSymbols = ref<string[]>(loadTrackedSymbols())
  const quoteHistory = ref<Record<string, QuotePoint[]>>(initialHistory)
  const timelineNotes = ref<Record<string, TimelineNote[]>>(loadTimelineNotes())
  const priceAlerts = ref<PriceAlert[]>(loadPriceAlerts())
  const { getQuote, getQuotes, getHistory, searchSymbols } = useApi()

  function persistHistory() {
    writeJson(HISTORY_KEY, quoteHistory.value)
  }

  function persistTrackedSymbols() {
    writeJson(TRACKED_SYMBOLS_KEY, trackedSymbols.value)
  }

  function persistTimelineNotes() {
    writeJson(TIMELINE_NOTES_KEY, timelineNotes.value)
  }

  function persistPriceAlerts() {
    writeJson(PRICE_ALERTS_KEY, priceAlerts.value)
  }

  function recordQuote(quote: Quote) {
    if (!Number.isFinite(quote.price) || quote.price <= 0) return

    const symbol = normalizeSymbol(quote.symbol)
    const point: QuotePoint = {
      symbol,
      price: quote.price,
      change: quote.change,
      change_percent: quote.change_percent,
      volume: quote.volume,
      time: quote.timestamp > 0 ? quote.timestamp * 1000 : Date.now(),
      session: quote.session,
    }
    const cutoff = Date.now() - MAX_HISTORY_AGE_MS
    const current = quoteHistory.value[symbol] ?? []
    const last = current[current.length - 1]
    const next = last && Math.abs(last.time - point.time) < 750
      ? [...current.slice(0, -1), point]
      : [...current, point]

    quoteHistory.value = {
      ...quoteHistory.value,
      [symbol]: next
        .filter((item) => item.time >= cutoff)
        .slice(-MAX_POINTS_PER_SYMBOL),
    }
    persistHistory()
  }

  function mergeHistoryPoints(symbol: string, points: QuotePoint[]) {
    const normalized = normalizeSymbol(symbol)
    if (!normalized || !points.length) return

    const cutoff = Date.now() - MAX_HISTORY_AGE_MS
    const byTime = new Map<number, QuotePoint>()
    for (const point of quoteHistory.value[normalized] ?? []) {
      if (point.time >= cutoff) byTime.set(point.time, point)
    }
    for (const point of points) {
      if (point.time >= cutoff) byTime.set(point.time, point)
    }

    const merged = Array.from(byTime.values())
      .sort((a, b) => a.time - b.time)
      .slice(-MAX_POINTS_PER_SYMBOL)

    quoteHistory.value = {
      ...quoteHistory.value,
      [normalized]: merged,
    }
    persistHistory()

    const latest = merged[merged.length - 1]
    if (latest && !quotes.value.has(normalized)) {
      quotes.value.set(normalized, quoteFromPoint(latest))
    }
  }

  function pointsFromBars(bars: HistoryBar[]): QuotePoint[] {
    const sorted = [...bars].sort((a, b) => a.time - b.time)
    return sorted
      .filter((bar) =>
        Number.isFinite(bar.close)
        && bar.close > 0
        && Number.isFinite(bar.time)
      )
      .map((bar, index) => {
        const previous = sorted[index - 1]
        const previousClose = previous?.close ?? bar.open
        const change = bar.close - previousClose
        const changePercent = previousClose ? (change / previousClose) * 100 : 0
        return {
          symbol: normalizeSymbol(bar.symbol),
          price: bar.close,
          change,
          change_percent: changePercent,
          volume: bar.volume,
          time: bar.time * 1000,
          session: bar.session,
        }
      })
  }

  function addTrackedSymbol(symbol: string) {
    const normalized = normalizeSymbol(symbol)
    if (!normalized || trackedSymbols.value.includes(normalized)) return

    trackedSymbols.value = [normalized, ...trackedSymbols.value].slice(0, 24)
    persistTrackedSymbols()
  }

  function removeTrackedSymbol(symbol: string) {
    const normalized = normalizeSymbol(symbol)
    trackedSymbols.value = trackedSymbols.value.filter((item) => item !== normalized)
    persistTrackedSymbols()
  }

  function checkQuoteAlerts(quote: Quote) {
    const symbol = normalizeSymbol(quote.symbol)
    const now = quote.timestamp > 0 ? quote.timestamp * 1000 : Date.now()
    let changed = false

    priceAlerts.value = priceAlerts.value.map((alert) => {
      if (alert.symbol !== symbol || !alert.enabled || alert.triggered_at) return alert
      const triggered = alert.direction === 'above'
        ? quote.price >= alert.target
        : quote.price <= alert.target
      if (!triggered) return alert

      changed = true
      return { ...alert, triggered_at: now }
    })

    if (changed) persistPriceAlerts()
  }

  async function fetchQuote(symbol: string) {
    const quote = await getQuote(symbol)
    quotes.value.set(normalizeSymbol(symbol), quote)
    recordQuote(quote)
    checkQuoteAlerts(quote)
    return quote
  }

  async function fetchQuotes(nextSymbols: string[]) {
    const uniqueSymbols = Array.from(
      new Set(
        nextSymbols
          .map(normalizeSymbol)
          .filter(Boolean),
      ),
    )
    if (!uniqueSymbols.length) return []

    const nextQuotes = await getQuotes(uniqueSymbols)
    for (const quote of nextQuotes) {
      quotes.value.set(quote.symbol.toUpperCase(), quote)
      recordQuote(quote)
      checkQuoteAlerts(quote)
    }
    return nextQuotes
  }

  async function fetchHistory(
    symbol: string,
    options: { interval: string; range: string; extendedHours?: boolean },
  ) {
    const normalized = normalizeSymbol(symbol)
    const bars = await getHistory(normalized, options)
    mergeHistoryPoints(normalized, pointsFromBars(bars))
    return bars
  }

  async function search(query: string) {
    symbols.value = await searchSymbols(query)
  }

  function historyFor(symbol: string) {
    return quoteHistory.value[normalizeSymbol(symbol)] ?? []
  }

  function clearHistory(symbol: string) {
    const key = normalizeSymbol(symbol)
    const { [key]: _removed, ...rest } = quoteHistory.value
    quoteHistory.value = rest
    persistHistory()
  }

  function notesFor(symbol: string) {
    return timelineNotes.value[normalizeSymbol(symbol)] ?? []
  }

  function addTimelineNote(symbol: string, time: number, price: number, text: string) {
    const normalized = normalizeSymbol(symbol)
    const cleaned = text.trim()
    if (!normalized || !cleaned || !Number.isFinite(time) || !Number.isFinite(price)) return undefined

    const note: TimelineNote = {
      id: createId('note'),
      symbol: normalized,
      time,
      price,
      text: cleaned,
      created_at: Date.now(),
    }

    timelineNotes.value = {
      ...timelineNotes.value,
      [normalized]: [note, ...notesFor(normalized)].slice(0, 120),
    }
    persistTimelineNotes()
    return note
  }

  function removeTimelineNote(symbol: string, id: string) {
    const normalized = normalizeSymbol(symbol)
    timelineNotes.value = {
      ...timelineNotes.value,
      [normalized]: notesFor(normalized).filter((note) => note.id !== id),
    }
    persistTimelineNotes()
  }

  function addPriceAlert(symbol: string, direction: PriceAlert['direction'], target: number) {
    const normalized = normalizeSymbol(symbol)
    if (!normalized || !Number.isFinite(target) || target <= 0) return undefined

    const alert: PriceAlert = {
      id: createId('alert'),
      symbol: normalized,
      direction,
      target,
      enabled: true,
      created_at: Date.now(),
    }
    priceAlerts.value = [alert, ...priceAlerts.value].slice(0, 100)
    persistPriceAlerts()
    return alert
  }

  function removePriceAlert(id: string) {
    priceAlerts.value = priceAlerts.value.filter((alert) => alert.id !== id)
    persistPriceAlerts()
  }

  function togglePriceAlert(id: string) {
    priceAlerts.value = priceAlerts.value.map((alert) =>
      alert.id === id ? { ...alert, enabled: !alert.enabled } : alert,
    )
    persistPriceAlerts()
  }

  function resetPriceAlert(id: string) {
    priceAlerts.value = priceAlerts.value.map((alert) =>
      alert.id === id ? { ...alert, triggered_at: undefined, enabled: true } : alert,
    )
    persistPriceAlerts()
  }

  return {
    quotes,
    symbols,
    trackedSymbols,
    quoteHistory,
    timelineNotes,
    priceAlerts,
    fetchQuote,
    fetchQuotes,
    fetchHistory,
    search,
    historyFor,
    clearHistory,
    addTrackedSymbol,
    removeTrackedSymbol,
    notesFor,
    addTimelineNote,
    removeTimelineNote,
    addPriceAlert,
    removePriceAlert,
    togglePriceAlert,
    resetPriceAlert,
  }
})
