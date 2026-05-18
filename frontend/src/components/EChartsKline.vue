<template>
  <div ref="containerRef" class="echart-kline">
    <div v-if="data.length < 1" class="echart-kline__empty">暂无K线数据</div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as echarts from 'echarts/core'
import type { EChartsCoreOption } from 'echarts/core'
import { CandlestickChart, BarChart, LineChart } from 'echarts/charts'
import {
  DataZoomComponent,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([
  CandlestickChart,
  BarChart,
  LineChart,
  GridComponent,
  TooltipComponent,
  DataZoomComponent,
  MarkLineComponent,
  CanvasRenderer,
])

export interface KlineData {
  time: number
  open: number
  high: number
  low: number
  close: number
  volume: number
}

const props = defineProps<{ data: KlineData[] }>()

const containerRef = ref<HTMLDivElement>()
let chart: echarts.ECharts | undefined
let resizeObserver: ResizeObserver | undefined

const UP_COLOR = '#18a957'
const DOWN_COLOR = '#ff5b5b'

function sma(values: number[], period: number): (number | null)[] {
  const result: (number | null)[] = []
  for (let i = 0; i < values.length; i++) {
    if (i < period - 1) { result.push(null); continue }
    let sum = 0
    for (let j = i - period + 1; j <= i; j++) sum += values[j]
    result.push(sum / period)
  }
  return result
}

function volumeLabel(v: number): string {
  if (v >= 1e9) return (v / 1e9).toFixed(2) + 'B'
  if (v >= 1e6) return (v / 1e6).toFixed(2) + 'M'
  if (v >= 1e3) return (v / 1e3).toFixed(0) + 'K'
  return v.toString()
}

function formatDate(ts: number, full?: boolean): string {
  const opts: Intl.DateTimeFormatOptions = {
    timeZone: 'America/New_York',
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    ...(full ? { hour: '2-digit', minute: '2-digit' } : {}),
  }
  return new Intl.DateTimeFormat('zh-CN', opts).format(new Date(ts))
}

function buildOption(): EChartsCoreOption {
  const raw = props.data
  if (!raw.length) return {}

  const closes = raw.map((d) => d.close)
  const ohlc = raw.map((d) => [d.open, d.close, d.low, d.high])
  const vols = raw.map((d, i) => [i, d.volume, d.close >= d.open ? 1 : -1])
  const timestamps = raw.map((d) => d.time)
  const ma5 = sma(closes, 5)
  const ma10 = sma(closes, 10)
  const ma20 = sma(closes, 20)
  const last = raw[raw.length - 1]

  return {
    animationDuration: 300,
    backgroundColor: 'transparent',
    textStyle: { fontFamily: 'Inter, system-ui, sans-serif' },
    grid: [
      { left: 60, right: 64, top: 24, bottom: '55%', height: 'auto' },
      { left: 60, right: 64, top: '59%', bottom: 64, height: 'auto' },
    ],
    xAxis: [
      {
        type: 'category', data: timestamps, gridIndex: 0,
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: { show: false }, splitLine: { show: false },
      },
      {
        type: 'category', data: timestamps, gridIndex: 1,
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: {
          color: '#8e8e9a', fontSize: 11,
          formatter: (v: unknown) => formatDate(v as number),
          interval: Math.max(1, Math.floor(timestamps.length / 5)),
        },
        splitLine: { show: false },
      },
    ],
    yAxis: [
      {
        type: 'value', gridIndex: 0, scale: true,
        position: 'left',
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: { color: '#8e8e9a', fontSize: 11, formatter: (v: unknown) => (v as number).toFixed(2) },
        splitLine: { lineStyle: { color: '#e0e4e9', width: 0.5 } },
        splitNumber: 4,
      },
      {
        type: 'value', gridIndex: 0, scale: true,
        position: 'right',
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: { show: false },
        splitLine: { show: false },
        splitNumber: 4,
      },
      {
        type: 'value', gridIndex: 1, scale: true,
        axisLine: { show: false }, axisTick: { show: false },
        axisLabel: {
          color: '#8e8e9a', fontSize: 10,
          formatter: (v: unknown) => volumeLabel(v as number),
        },
        splitLine: { show: false },
        position: 'left',
      },
    ],
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'cross' },
      backgroundColor: 'rgba(26, 26, 46, 0.94)',
      borderColor: 'transparent',
      textStyle: { color: '#fff', fontSize: 12 },
      formatter: (params: unknown) => {
        const items = params as Array<{ axisValue: number; seriesName: string; data: number[] }>
        if (!items?.length) return ''
        const t = items[0].axisValue
        const c = items.find((i) => i.seriesName === 'K线')?.data
        const v = items.find((i) => i.seriesName === 'VOL')?.data
        if (!c) return ''
        const isUp = c[2] >= c[1]
        const sign = isUp ? '+' : ''
        const color = isUp ? UP_COLOR : DOWN_COLOR
        return `<div style="color:#8e8e9a;margin-bottom:4px">${formatDate(t, true)}</div>
          <table style="border-spacing:0">
            <tr><td style="color:#8e8e9a">开</td><td style="font-weight:700;text-align:right;padding-left:8px">${c[1]?.toFixed(2)}</td></tr>
            <tr><td style="color:#8e8e9a">高</td><td style="font-weight:700;text-align:right;color:${UP_COLOR};padding-left:8px">${c[4]?.toFixed(2)}</td></tr>
            <tr><td style="color:#8e8e9a">低</td><td style="font-weight:700;text-align:right;color:${DOWN_COLOR};padding-left:8px">${c[3]?.toFixed(2)}</td></tr>
            <tr><td style="color:#8e8e9a">收</td><td style="font-weight:700;text-align:right;color:${color};padding-left:8px">${c[2]?.toFixed(2)}</td></tr>
            <tr><td style="color:#8e8e9a">幅</td><td style="font-weight:700;text-align:right;color:${color};padding-left:8px">${sign}${(c[2] >= c[1] ? (c[2]-c[1])/c[1]*100 : (c[1]-c[2])/c[1]*100).toFixed(2)}%</td></tr>
            <tr><td style="color:#8e8e9a">量</td><td style="font-weight:700;text-align:right;padding-left:8px">${v ? volumeLabel(v[1]) : '--'}</td></tr>
          </table>`
      },
    },
    dataZoom: [
      {
        type: 'slider', height: 22, bottom: 12, xAxisIndex: [0, 1],
        backgroundColor: 'rgba(200,205,211,0.30)',
        fillerColor: 'rgba(91,127,255,0.14)',
        dataBackground: {
          lineStyle: { color: '#5b7fff', opacity: 0.24, width: 1 },
          areaStyle: { color: '#5b7fff', opacity: 0.06 },
        },
        handleStyle: { color: '#5b7fff', borderColor: '#5b7fff' },
        textStyle: { color: '#8e8e9a', fontSize: 10 },
        moveHandleSize: 5, handleSize: '75%',
      },
      { type: 'inside', xAxisIndex: [0, 1] },
    ],
    series: [
      {
        name: 'K线', type: 'candlestick', data: ohlc,
        xAxisIndex: 0, yAxisIndex: 0,
        itemStyle: {
          color: UP_COLOR, color0: DOWN_COLOR,
          borderColor: UP_COLOR, borderColor0: DOWN_COLOR,
        },
        markLine: {
          silent: true, symbol: 'none',
          lineStyle: { color: '#8e8e9a', type: 'dashed', width: 1, opacity: 0.6 },
          label: { show: true, position: 'end', color: '#8e8e9a', fontSize: 11, formatter: `$ ${last.close.toFixed(2)}` },
          data: [{ yAxis: last.close }],
        },
      },
      {
        name: 'MA5', type: 'line', data: ma5,
        xAxisIndex: 0, yAxisIndex: 0,
        showSymbol: false, lineStyle: { width: 1.2, color: '#f5a623' },
        silent: true, z: 2,
      },
      {
        name: 'MA10', type: 'line', data: ma10,
        xAxisIndex: 0, yAxisIndex: 0,
        showSymbol: false, lineStyle: { width: 1.2, color: '#7b61ff' },
        silent: true, z: 2,
      },
      {
        name: 'MA20', type: 'line', data: ma20,
        xAxisIndex: 0, yAxisIndex: 0,
        showSymbol: false, lineStyle: { width: 1.2, color: '#4a90d9' },
        silent: true, z: 2,
      },
      {
        name: 'VOL', type: 'bar', data: vols,
        xAxisIndex: 1, yAxisIndex: 2,
        itemStyle: {
          color: (p: unknown) => (p as { data: number[] }).data[2] === 1 ? UP_COLOR : DOWN_COLOR,
        },
        silent: true,
      },
    ],
  }
}

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
watch(() => props.data, updateChart, { deep: true })
onBeforeUnmount(() => { resizeObserver?.disconnect(); chart?.dispose() })
</script>

<style scoped>
.echart-kline {
  position: relative; width: 100%; min-height: 440px;
  border-radius: var(--neu-radius-sm);
  background: linear-gradient(145deg, rgba(255,255,255,0.34), rgba(223,227,232,0.76));
  box-shadow: inset 3px 3px 8px var(--neu-shadow-dark), inset -3px -3px 8px var(--neu-shadow-light);
  overflow: hidden;
}
.echart-kline__empty {
  position: absolute; inset: 0; display: grid; place-items: center;
  color: var(--neu-text-dim); font-size: 14px; pointer-events: none; z-index: 1;
}
</style>
