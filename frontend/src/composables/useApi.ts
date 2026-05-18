import axios from 'axios'
import type { HistoryBar, PortfolioHolding, Quote, WatchlistItem } from '../types'

const api = axios.create({
  baseURL: '/api/v1',
  timeout: 10000,
})

export function useApi() {
  async function getQuote(symbol: string): Promise<Quote> {
    const { data } = await api.get<Quote>(`/market/quote/${symbol}`)
    return data
  }

  async function getQuotes(symbols: string[]): Promise<Quote[]> {
    const encodedSymbols = symbols.map((symbol) => encodeURIComponent(symbol)).join(',')
    const { data } = await api.get<Quote[]>(`/market/quotes/${encodedSymbols}`)
    return data
  }

  async function getHistory(
    symbol: string,
    options: { interval: string; range: string; extendedHours?: boolean },
  ): Promise<HistoryBar[]> {
    const { data } = await api.get<HistoryBar[]>(`/market/history/${encodeURIComponent(symbol)}`, {
      params: {
        interval: options.interval,
        range: options.range,
        extended_hours: options.extendedHours ?? true,
      },
    })
    return data
  }

  async function searchSymbols(query: string): Promise<string[]> {
    const { data } = await api.get<string[]>(`/market/search/${query}`)
    return data
  }

  async function getWatchlist(): Promise<WatchlistItem[]> {
    const { data } = await api.get<WatchlistItem[]>('/portfolio/watchlist')
    return data
  }

  async function getHoldings(): Promise<PortfolioHolding[]> {
    const { data } = await api.get<PortfolioHolding[]>('/portfolio/holdings')
    return data
  }

  return { getQuote, getQuotes, getHistory, searchSymbols, getWatchlist, getHoldings }
}
