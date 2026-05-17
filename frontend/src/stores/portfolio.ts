import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { PortfolioHolding, WatchlistItem } from '../types'
import { useApi } from '../composables/useApi'

export const usePortfolioStore = defineStore('portfolio', () => {
  const watchlist = ref<WatchlistItem[]>([])
  const holdings = ref<PortfolioHolding[]>([])
  const { getHoldings, getWatchlist } = useApi()

  async function fetchWatchlist() {
    watchlist.value = await getWatchlist()
  }

  async function fetchHoldings() {
    holdings.value = await getHoldings()
  }

  async function fetchPortfolio() {
    await Promise.all([fetchWatchlist(), fetchHoldings()])
  }

  return { watchlist, holdings, fetchWatchlist, fetchHoldings, fetchPortfolio }
})
