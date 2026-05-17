export interface Quote {
  symbol: string
  price: number
  change: number
  change_percent: number
  volume: number
  timestamp: number
}

export interface QuotePoint {
  symbol: string
  price: number
  change: number
  change_percent: number
  volume: number
  time: number
}

export interface WatchlistItem {
  id: string
  user_id: string
  symbol: string
  added_at: string
}

export interface PortfolioHolding {
  id: string
  user_id: string
  symbol: string
  quantity: number
  avg_cost: number
  updated_at: string
}

export type CurrencyCode = 'USD' | 'CNY' | 'HKD'

export interface ExchangeRates {
  base: 'USD'
  date: string
  rates: Record<CurrencyCode, number>
}

export interface ExchangeRatePoint {
  date: string
  time: number
  usdCny: number
  usdHkd: number
}

export interface CashBalances {
  USD: number
  CNY: number
  HKD: number
}

export interface Kline {
  id: string
  symbol: string
  interval: string
  open_time: string
  open: number
  high: number
  low: number
  close: number
  volume: number
}
