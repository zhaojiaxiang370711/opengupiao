import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Dashboard',
      component: () => import('../pages/Dashboard.vue'),
    },
    {
      path: '/market',
      name: 'Market',
      component: () => import('../pages/Market.vue'),
    },
    {
      path: '/portfolio',
      name: 'Portfolio',
      component: () => import('../pages/Portfolio.vue'),
    },
    {
      path: '/exchange',
      name: 'ExchangeRates',
      component: () => import('../pages/ExchangeRates.vue'),
    },
    {
      path: '/assets',
      name: 'Assets',
      component: () => import('../pages/Assets.vue'),
    },
    {
      path: '/learn',
      name: 'Learning',
      component: () => import('../pages/Learning.vue'),
    },
  ],
})

export default router
