import type { CurrencyCode, ExchangeRatePoint, ExchangeRates } from '../types'

interface FrankfurterLatest {
  base: string
  date: string
  rates: Partial<Record<CurrencyCode, number>>
}

interface FrankfurterSeries {
  rates: Record<string, Partial<Record<CurrencyCode, number>>>
}

const API_BASE = 'https://api.frankfurter.dev/v1'

const fallbackRates: ExchangeRates = {
  base: 'USD',
  date: new Date().toISOString().slice(0, 10),
  rates: {
    USD: 1,
    CNY: 7.1,
    HKD: 7.8,
  },
}

export function currencySymbol(currency: CurrencyCode): string {
  if (currency === 'CNY') return '¥'
  if (currency === 'HKD') return 'HK$'
  return '$'
}

export function currencyName(currency: CurrencyCode): string {
  if (currency === 'CNY') return '人民币'
  if (currency === 'HKD') return '港币'
  return '美元'
}

export function convertCurrency(
  amount: number,
  from: CurrencyCode,
  to: CurrencyCode,
  rates: ExchangeRates,
): number {
  if (from === to) return amount
  const fromRate = rates.rates[from] || 1
  const toRate = rates.rates[to] || 1
  const amountInUsd = amount / fromRate
  return amountInUsd * toRate
}

export function formatCurrency(amount: number, currency: CurrencyCode): string {
  return `${currencySymbol(currency)}${amount.toLocaleString('zh-CN', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 2,
  })}`
}

export function pairRate(
  from: CurrencyCode,
  to: CurrencyCode,
  rates: ExchangeRates,
): number {
  return convertCurrency(1, from, to, rates)
}

export async function fetchLatestRates(): Promise<ExchangeRates> {
  const response = await fetch(`${API_BASE}/latest?from=USD&to=CNY,HKD`)
  if (!response.ok) throw new Error(`汇率同步失败: ${response.status}`)
  const data = await response.json() as FrankfurterLatest

  return {
    base: 'USD',
    date: data.date,
    rates: {
      USD: 1,
      CNY: data.rates.CNY ?? fallbackRates.rates.CNY,
      HKD: data.rates.HKD ?? fallbackRates.rates.HKD,
    },
  }
}

export async function fetchRateHistory(days = 60): Promise<ExchangeRatePoint[]> {
  const end = new Date()
  const start = new Date(end)
  start.setDate(start.getDate() - days)
  const startDate = start.toISOString().slice(0, 10)
  const response = await fetch(`${API_BASE}/${startDate}..?from=USD&to=CNY,HKD`)
  if (!response.ok) throw new Error(`汇率历史同步失败: ${response.status}`)
  const data = await response.json() as FrankfurterSeries

  return Object.entries(data.rates)
    .map(([date, rates]) => ({
      date,
      time: new Date(`${date}T00:00:00Z`).getTime(),
      usdCny: rates.CNY ?? 0,
      usdHkd: rates.HKD ?? 0,
    }))
    .filter((point) => point.usdCny > 0 && point.usdHkd > 0)
    .sort((a, b) => a.time - b.time)
}

export function defaultExchangeRates(): ExchangeRates {
  return fallbackRates
}
