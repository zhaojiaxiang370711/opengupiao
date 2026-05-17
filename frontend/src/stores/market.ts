import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Quote, QuotePoint } from '../types'
import { useApi } from '../composables/useApi'

const HISTORY_KEY = 'aaagupiao.quoteHistory.v1'
const MAX_POINTS_PER_SYMBOL = 5000
const MAX_HISTORY_AGE_MS = 30 * 24 * 60 * 60 * 1000

function loadHistory(): Record<string, QuotePoint[]> {
  if (typeof localStorage === 'undefined') return {}

  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw) as Record<string, QuotePoint[]>
    const cutoff = Date.now() - MAX_HISTORY_AGE_MS

    return Object.fromEntries(
      Object.entries(parsed).map(([symbol, points]) => [
        symbol,
        points
          .filter((point) => Number.isFinite(point.price) && point.price > 0 && point.time >= cutoff)
          .slice(-MAX_POINTS_PER_SYMBOL),
      ]),
    )
  } catch {
    return {}
  }
}

function quoteFromPoint(point: QuotePoint): Quote {
  return {
    symbol: point.symbol.toUpperCase(),
    price: point.price,
    change: point.change,
    change_percent: point.change_percent,
    volume: point.volume,
    timestamp: Math.floor(point.time / 1000),
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
  const quoteHistory = ref<Record<string, QuotePoint[]>>(initialHistory)
  const { getQuote, getQuotes, searchSymbols } = useApi()

  function persistHistory() {
    if (typeof localStorage === 'undefined') return
    localStorage.setItem(HISTORY_KEY, JSON.stringify(quoteHistory.value))
  }

  function recordQuote(quote: Quote) {
    if (!Number.isFinite(quote.price) || quote.price <= 0) return

    const symbol = quote.symbol.toUpperCase()
    const point: QuotePoint = {
      symbol,
      price: quote.price,
      change: quote.change,
      change_percent: quote.change_percent,
      volume: quote.volume,
      time: quote.timestamp > 0 ? quote.timestamp * 1000 : Date.now(),
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

  async function fetchQuote(symbol: string) {
    const quote = await getQuote(symbol)
    quotes.value.set(symbol.toUpperCase(), quote)
    recordQuote(quote)
    return quote
  }

  async function fetchQuotes(nextSymbols: string[]) {
    const uniqueSymbols = Array.from(
      new Set(
        nextSymbols
          .map((symbol) => symbol.trim().toUpperCase())
          .filter(Boolean),
      ),
    )
    if (!uniqueSymbols.length) return []

    const nextQuotes = await getQuotes(uniqueSymbols)
    for (const quote of nextQuotes) {
      quotes.value.set(quote.symbol.toUpperCase(), quote)
      recordQuote(quote)
    }
    return nextQuotes
  }

  async function search(query: string) {
    symbols.value = await searchSymbols(query)
  }

  function historyFor(symbol: string) {
    return quoteHistory.value[symbol.toUpperCase()] ?? []
  }

  function clearHistory(symbol: string) {
    const key = symbol.toUpperCase()
    const { [key]: _removed, ...rest } = quoteHistory.value
    quoteHistory.value = rest
    persistHistory()
  }

  return { quotes, symbols, quoteHistory, fetchQuote, fetchQuotes, search, historyFor, clearHistory }
})
