import axios from 'axios'
import type { PortfolioHolding, Quote, WatchlistItem } from '../types'

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

  return { getQuote, getQuotes, searchSymbols, getWatchlist, getHoldings }
}
