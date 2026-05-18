<template>
  <div ref="containerRef" class="echart-line">
    <div v-if="points.length < 2" class="echart-line__empty">采集中</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as echarts from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  DataZoomComponent,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
  MarkPointComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([
  LineChart,
  GridComponent,
  TooltipComponent,
  DataZoomComponent,
  MarkLineComponent,
  MarkPointComponent,
  CanvasRenderer,
])

const MARKET_TZ = 'America/New_York'

const props = defineProps<{
  points: Array<{ time: number; price: number }>
  accent?: 'up' | 'down' | 'flat'
}>()

const containerRef = ref<HTMLDivElement>()
let chart: echarts.ECharts | undefined
let resizeObserver: ResizeObserver | undefined

const strokeColor = computed(() => {
  if (props.accent === 'down') return '#ff5b5b'
  if (props.accent === 'up') return '#18a957'
  return '#5b7fff'
})

function fmtAxis(ts: number, spanMs: number): string {
  const d = new Date(ts)
  const b: Intl.DateTimeFormatOptions = { timeZone: MARKET_TZ, hour12: false }
  if (spanMs > 7 * 864e5) return new Intl.DateTimeFormat('zh-CN', { ...b, month: '2-digit', day: '2-digit' }).format(d)
  if (spanMs > 864e5) return new Intl.DateTimeFormat('zh-CN', { ...b, month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(d)
  return new Intl.DateTimeFormat('zh-CN', { ...b, hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(d)
}

function fmtTooltip(ts: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    timeZone: MARKET_TZ, hour12: false,
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
  }).format(new Date(ts))
}

function buildOption(): echarts.EChartsOption {
  const pts = props.points
  if (!pts.length) return {}

  const prices = pts.map((p) => p.price)
  const dataMin = Math.min(...prices)
  const dataMax = Math.max(...prices)
  const range = dataMax - dataMin || Math.max(dataMax * 0.001, 0.01)
  const yMin = dataMin - range * 0.08
  const yMax = dataMax + range * 0.08
  const first = pts[0]
  const last = pts[pts.length - 1]
  const ms = Math.max(0, last.time - first.time)
  const chg = last.price - first.price
  const chgPct = first.price !== 0 ? (chg / first.price) * 100 : 0

  return {
    animationDuration: 200,
    animationEasing: 'linear',
    backgroundColor: 'transparent',
    textStyle: { fontFamily: 'Inter, system-ui, sans-serif' },
    grid: { left: 60, right: 64, top: 24, bottom: 64 },
    xAxis: {
      type: 'time',
      axisLine: { show: false }, axisTick: { show: false },
      axisLabel: {
        color: '#8e8e9a', fontSize: 11,
        formatter: (v: unknown) => fmtAxis(v as number, ms),
        showMaxLabel: true, showMinLabel: true,
      },
      splitLine: { show: false },
      min: first.time, max: last.time,
    },
    yAxis: [
      {
        type: 'value', min: yMin, max: yMax,
        position: 'left',
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: { color: '#8e8e9a', fontSize: 11, formatter: (v: unknown) => (v as number).toFixed(2) },
        splitLine: { lineStyle: { color: '#e0e4e9', width: 0.5 } },
        splitNumber: 4,
      },
      {
        type: 'value', min: yMin, max: yMax,
        position: 'right',
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: { show: false },
        splitLine: { show: false },
        splitNumber: 4,
      },
    ],
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(26,26,46,0.94)',
      borderColor: 'transparent',
      textStyle: { color: '#fff', fontSize: 12 },
      formatter: (params: unknown) => {
        const items = params as Array<{ data: number[] }>
        if (!items?.length) return ''
        const [t, price] = items[0].data
        const chgFromFirst = first.price !== 0 ? ((price - first.price) / first.price * 100) : 0
        const sign = chgFromFirst >= 0 ? '+' : ''
        const c = chgFromFirst >= 0 ? UP : DOWN
        return `<div style="color:#8e8e9a;margin-bottom:2px">${fmtTooltip(t)}</div>
          <span style="font-weight:700;font-size:16px">$${price.toFixed(2)}</span>
          <span style="margin-left:8px;font-weight:700;font-size:13px;color:${c}">${sign}${chgFromFirst.toFixed(2)}%</span>`
      },
      axisPointer: {
        type: 'cross',
        crossStyle: { color: '#8e8e9a' },
        label: { show: false },
      },
    },
    dataZoom: [
      {
        type: 'slider', height: 22, bottom: 12,
        backgroundColor: 'rgba(200,205,211,0.30)',
        fillerColor: 'rgba(91,127,255,0.14)',
        dataBackground: {
          lineStyle: { color: strokeColor.value, opacity: 0.24, width: 1 },
          areaStyle: { color: strokeColor.value, opacity: 0.06 },
        },
        handleStyle: { color: strokeColor.value, borderColor: strokeColor.value },
        textStyle: { color: '#8e8e9a', fontSize: 10 },
        moveHandleSize: 5, handleSize: '75%',
      },
      { type: 'inside' },
    ],
    series: [
      {
        type: 'line',
        data: pts.map((p) => [p.time, p.price]),
        yAxisIndex: 0,
        showSymbol: false,
        lineStyle: { color: strokeColor.value, width: 2.5 },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: `${strokeColor.value}28` },
            { offset: 1, color: `${strokeColor.value}00` },
          ]),
        },
        markLine: {
          silent: true, symbol: 'none',
          lineStyle: { color: strokeColor.value, type: 'dashed', width: 1, opacity: 0.7 },
          label: {
            show: true, position: 'end', color: strokeColor.value, fontSize: 12, fontWeight: 700,
            formatter: `$ ${last.price.toFixed(2)}`,
          },
          data: [{ yAxis: last.price }],
          animation: false,
        },
        markPoint: {
          data: [{
            name: '最新',
            coord: [last.time, last.price],
            symbol: 'circle', symbolSize: 8,
            itemStyle: { color: strokeColor.value, borderColor: '#fff', borderWidth: 2 },
            label: { show: false },
          }],
          animation: false,
        },
      },
    ],
  } as echarts.EChartsOption
}

const UP = '#18a957'
const DOWN = '#ff5b5b'

function initChart() {
  if (!containerRef.value) return
  chart = echarts.init(containerRef.value)
  chart.setOption(buildOption(), { notMerge: true })
  resizeObserver = new ResizeObserver(() => chart?.resize())
  resizeObserver.observe(containerRef.value)
}

function updateChart() {
  if (!chart) return
  chart.setOption(buildOption(), { notMerge: false, lazyUpdate: true })
}

onMounted(initChart)
watch(() => [props.points, props.accent], updateChart, { deep: true })
onBeforeUnmount(() => { resizeObserver?.disconnect(); chart?.dispose() })
</script>

<style scoped>
.echart-line {
  position: relative; width: 100%; min-height: 320px;
  border-radius: var(--neu-radius-sm);
  background: linear-gradient(145deg, rgba(255,255,255,0.34), rgba(223,227,232,0.76));
  box-shadow: inset 3px 3px 8px var(--neu-shadow-dark), inset -3px -3px 8px var(--neu-shadow-light);
  overflow: hidden;
}
.echart-line__empty {
  position: absolute; inset: 0; display: grid; place-items: center;
  color: var(--neu-text-dim); font-size: 14px; pointer-events: none; z-index: 1;
}
</style>
