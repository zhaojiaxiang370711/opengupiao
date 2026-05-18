<template>
  <div ref="containerRef" class="echart-line">
    <div v-if="points.length < 2" class="echart-line__empty">采集中</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as echarts from 'echarts/core'
import type { EChartsCoreOption } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  MarkLineComponent,
  MarkPointComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import type { TimelineMarker } from '../types'

echarts.use([
  LineChart,
  GridComponent,
  TooltipComponent,
  DataZoomComponent,
  LegendComponent,
  MarkLineComponent,
  MarkPointComponent,
  CanvasRenderer,
])

interface ChartPoint {
  time: number
  price: number
}

const MARKET_TZ = 'America/New_York'
const UP = '#18a957'
const DOWN = '#ff5b5b'
const INFO = '#5b7fff'
const WARNING = '#f5a623'
const NEUTRAL = '#8e8e9a'

const props = withDefaults(defineProps<{
  points: ChartPoint[]
  accent?: 'up' | 'down' | 'flat'
  markers?: TimelineMarker[]
  selectedTime?: number
  baselinePrice?: number
}>(), {
  markers: () => [],
})

const emit = defineEmits<{
  markerSelect: [marker: TimelineMarker]
}>()

const containerRef = ref<HTMLDivElement>()
let chart: echarts.ECharts | undefined
let resizeObserver: ResizeObserver | undefined

const strokeColor = computed(() => {
  if (props.accent === 'down') return DOWN
  if (props.accent === 'up') return UP
  return INFO
})

function markerColor(marker: TimelineMarker): string {
  if (marker.tone === 'up') return UP
  if (marker.tone === 'down') return DOWN
  if (marker.tone === 'warning') return WARNING
  if (marker.tone === 'info') return INFO
  return NEUTRAL
}

function fmtAxis(ts: number, spanMs: number): string {
  const d = new Date(ts)
  const b: Intl.DateTimeFormatOptions = { timeZone: MARKET_TZ, hour12: false }
  if (spanMs > 7 * 864e5) return new Intl.DateTimeFormat('zh-CN', { ...b, month: '2-digit', day: '2-digit' }).format(d)
  if (spanMs > 864e5) return new Intl.DateTimeFormat('zh-CN', { ...b, month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }).format(d)
  return new Intl.DateTimeFormat('zh-CN', { ...b, hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(d)
}

function fmtTooltip(ts: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    timeZone: MARKET_TZ,
    hour12: false,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(ts))
}

function buildOption(): EChartsCoreOption {
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
  const markers = props.markers.filter((marker) =>
    marker.time >= first.time && marker.time <= last.time,
  )

  return {
    animationDuration: 200,
    animationEasing: 'linear',
    backgroundColor: 'transparent',
    textStyle: { fontFamily: 'Inter, system-ui, sans-serif' },
    grid: { left: 60, right: 64, top: 24, bottom: 64 },
    xAxis: {
      type: 'time',
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: {
        color: NEUTRAL,
        fontSize: 11,
        formatter: (v: unknown) => fmtAxis(v as number, ms),
        showMaxLabel: true,
        showMinLabel: true,
      },
      splitLine: { show: false },
      min: first.time,
      max: last.time,
    },
    yAxis: [
      {
        type: 'value',
        min: yMin,
        max: yMax,
        position: 'left',
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: { color: NEUTRAL, fontSize: 11, formatter: (v: unknown) => (v as number).toFixed(2) },
        splitLine: { lineStyle: { color: '#e0e4e9', width: 0.5 } },
        splitNumber: 4,
      },
      {
        type: 'value',
        min: yMin,
        max: yMax,
        position: 'right',
        axisLine: { show: false },
        axisTick: { show: false },
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
        const chgFromFirst = first.price !== 0 ? ((price - first.price) / first.price) * 100 : 0
        const sign = chgFromFirst >= 0 ? '+' : ''
        const color = chgFromFirst >= 0 ? UP : DOWN
        return `<div style="color:${NEUTRAL};margin-bottom:2px">${fmtTooltip(t)}</div>
          <span style="font-weight:700;font-size:16px">$${price.toFixed(2)}</span>
          <span style="margin-left:8px;font-weight:700;font-size:13px;color:${color}">${sign}${chgFromFirst.toFixed(2)}%</span>`
      },
      axisPointer: {
        type: 'cross',
        crossStyle: { color: NEUTRAL },
        label: { show: false },
      },
    },
    dataZoom: [
      {
        type: 'slider',
        height: 22,
        bottom: 12,
        backgroundColor: 'rgba(200,205,211,0.30)',
        fillerColor: 'rgba(91,127,255,0.14)',
        dataBackground: {
          lineStyle: { color: strokeColor.value, opacity: 0.24, width: 1 },
          areaStyle: { color: strokeColor.value, opacity: 0.06 },
        },
        handleStyle: { color: strokeColor.value, borderColor: strokeColor.value },
        textStyle: { color: NEUTRAL, fontSize: 10 },
        moveHandleSize: 5,
        handleSize: '75%',
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
          silent: true,
          symbol: 'none',
          lineStyle: { color: strokeColor.value, type: 'dashed', width: 1, opacity: 0.7 },
          label: {
            show: true,
            position: 'end',
            color: strokeColor.value,
            fontSize: 12,
            fontWeight: 700,
            formatter: `$ ${last.price.toFixed(2)}`,
          },
          data: [
            { yAxis: last.price },
            ...(Number.isFinite(props.baselinePrice)
              ? [{
                yAxis: props.baselinePrice,
                lineStyle: { color: NEUTRAL, type: 'dotted', opacity: 0.5 },
                label: { show: true, formatter: '基准', color: NEUTRAL },
              }]
              : []),
            ...(Number.isFinite(props.selectedTime)
              ? [{
                xAxis: props.selectedTime,
                lineStyle: { color: WARNING, type: 'solid', opacity: 0.58 },
                label: { show: false },
              }]
              : []),
          ],
          animation: false,
        },
        markPoint: {
          data: [
            {
              name: 'latest',
              coord: [last.time, last.price],
              symbol: 'circle',
              symbolSize: 8,
              itemStyle: { color: strokeColor.value, borderColor: '#fff', borderWidth: 2 },
              label: { show: false },
            },
            ...markers.map((marker) => ({
              name: marker.id,
              coord: [marker.time, marker.price],
              symbol: marker.kind === 'note' ? 'pin' : 'circle',
              symbolSize: marker.kind === 'note' ? 18 : 12,
              itemStyle: { color: markerColor(marker), borderColor: '#fff', borderWidth: 2 },
              label: { show: false },
              tooltip: {
                formatter: `<strong>${marker.title}</strong><br/>${marker.detail}`,
              },
            })),
          ],
          animation: false,
        },
      },
    ],
  }
}

function bindChartEvents() {
  if (!chart) return
  chart.off('click')
  chart.on('click', (params: { componentType?: string; name?: string }) => {
    if (params.componentType !== 'markPoint' || !params.name) return
    const marker = props.markers.find((item) => item.id === params.name)
    if (marker) emit('markerSelect', marker)
  })
}

function initChart() {
  if (!containerRef.value) return
  chart = echarts.init(containerRef.value)
  chart.setOption(buildOption(), { notMerge: true })
  bindChartEvents()
  resizeObserver = new ResizeObserver(() => chart?.resize())
  resizeObserver.observe(containerRef.value)
}

function updateChart() {
  if (!chart) return
  chart.setOption(buildOption(), { notMerge: false, lazyUpdate: true })
  bindChartEvents()
}

onMounted(initChart)
watch(() => [props.points, props.accent, props.markers, props.selectedTime, props.baselinePrice], updateChart, { deep: true })
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
