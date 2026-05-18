export interface Quote {
  symbol: string
  price: number
  change: number
  change_percent: number
  volume: number
  timestamp: number
  session: 'regular' | 'pre_market' | 'post_market' | string
  market_state?: string
  quote_source?: string
  regular_price?: number
  regular_change?: number
  regular_change_percent?: number
  regular_timestamp?: number
  pre_market_price?: number
  pre_market_change?: number
  pre_market_change_percent?: number
  pre_market_timestamp?: number
  post_market_price?: number
  post_market_change?: number
  post_market_change_percent?: number
  post_market_timestamp?: number
}

export interface QuotePoint {
  symbol: string
  price: number
  change: number
  change_percent: number
  volume: number
  time: number
  session?: string
}

export interface HistoryBar {
  symbol: string
  interval: string
  time: number
  open: number
  high: number
  low: number
  close: number
  volume: number
  session: 'regular' | 'pre_market' | 'post_market' | string
  source: string
}

export type TimelineMarkerKind =
  | 'range'
  | 'high'
  | 'low'
  | 'move'
  | 'volume'
  | 'session'
  | 'note'
  | 'alert'

export interface TimelineMarker {
  id: string
  symbol: string
  time: number
  price: number
  kind: TimelineMarkerKind
  title: string
  detail: string
  tone: 'up' | 'down' | 'neutral' | 'info' | 'warning'
}

export interface TimelineNote {
  id: string
  symbol: string
  time: number
  price: number
  text: string
  created_at: number
}

export interface PriceAlert {
  id: string
  symbol: string
  direction: 'above' | 'below'
  target: number
  enabled: boolean
  created_at: number
  triggered_at?: number
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
