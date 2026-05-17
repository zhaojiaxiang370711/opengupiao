<template>
  <div class="price-chart">
    <canvas ref="canvasRef" class="price-chart__canvas"></canvas>
    <div v-if="points.length < 2" class="price-chart__empty">采集中</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps<{
  points: Array<{ time: number; price: number }>
  accent?: 'up' | 'down' | 'flat'
}>()

const canvasRef = ref<HTMLCanvasElement>()
let resizeObserver: ResizeObserver | undefined

const strokeColor = computed(() => {
  if (props.accent === 'down') return '#ff5b5b'
  if (props.accent === 'up') return '#18a957'
  return '#5b7fff'
})

function draw() {
  const canvas = canvasRef.value
  if (!canvas) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const rect = canvas.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1
  const width = Math.max(1, rect.width)
  const height = Math.max(1, rect.height)

  canvas.width = Math.floor(width * dpr)
  canvas.height = Math.floor(height * dpr)
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, width, height)

  const padding = { top: 18, right: 16, bottom: 28, left: 58 }
  const plotWidth = width - padding.left - padding.right
  const plotHeight = height - padding.top - padding.bottom

  if (plotWidth <= 0 || plotHeight <= 0) return
  if (!props.points.length) return

  const prices = props.points.map((point) => point.price)
  const min = Math.min(...prices)
  const max = Math.max(...prices)
  const range = max - min || Math.max(max * 0.001, 0.01)
  const lower = min - range * 0.08
  const upper = max + range * 0.08
  const scaleY = (value: number) =>
    padding.top + ((upper - value) / (upper - lower)) * plotHeight
  const scaleX = (index: number) =>
    padding.left + (props.points.length <= 1 ? 0 : (index / (props.points.length - 1)) * plotWidth)

  ctx.lineWidth = 1
  ctx.strokeStyle = 'rgba(142, 142, 154, 0.28)'
  ctx.fillStyle = '#8e8e9a'
  ctx.font = '11px Inter, system-ui, sans-serif'
  ctx.textBaseline = 'middle'

  for (let i = 0; i <= 4; i += 1) {
    const y = padding.top + (i / 4) * plotHeight
    const price = upper - (i / 4) * (upper - lower)
    ctx.beginPath()
    ctx.moveTo(padding.left, y)
    ctx.lineTo(width - padding.right, y)
    ctx.stroke()
    ctx.fillText(price.toFixed(2), 8, y)
  }

  if (props.points.length < 2) return

  const line = new Path2D()
  props.points.forEach((point, index) => {
    const x = scaleX(index)
    const y = scaleY(point.price)
    if (index === 0) line.moveTo(x, y)
    else line.lineTo(x, y)
  })

  const area = new Path2D(line)
  area.lineTo(scaleX(props.points.length - 1), padding.top + plotHeight)
  area.lineTo(scaleX(0), padding.top + plotHeight)
  area.closePath()

  const gradient = ctx.createLinearGradient(0, padding.top, 0, padding.top + plotHeight)
  gradient.addColorStop(0, `${strokeColor.value}33`)
  gradient.addColorStop(1, `${strokeColor.value}00`)
  ctx.fillStyle = gradient
  ctx.fill(area)

  ctx.strokeStyle = strokeColor.value
  ctx.lineWidth = 2.5
  ctx.lineJoin = 'round'
  ctx.lineCap = 'round'
  ctx.stroke(line)

  const last = props.points[props.points.length - 1]
  const lastX = scaleX(props.points.length - 1)
  const lastY = scaleY(last.price)
  ctx.fillStyle = strokeColor.value
  ctx.beginPath()
  ctx.arc(lastX, lastY, 4, 0, Math.PI * 2)
  ctx.fill()
}

onMounted(() => {
  draw()
  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(draw)
    resizeObserver.observe(canvasRef.value)
  }
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})

watch(() => [props.points, props.accent], draw, { deep: true })
</script>

<style scoped>
.price-chart {
  position: relative;
  width: 100%;
  min-height: 320px;
  border-radius: var(--neu-radius-sm);
  background: linear-gradient(145deg, rgba(255, 255, 255, 0.34), rgba(223, 227, 232, 0.76));
  box-shadow:
    inset 3px 3px 8px var(--neu-shadow-dark),
    inset -3px -3px 8px var(--neu-shadow-light);
  overflow: hidden;
}

.price-chart__canvas {
  width: 100%;
  height: 100%;
  min-height: 320px;
  display: block;
}

.price-chart__empty {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: var(--neu-text-dim);
  font-size: 14px;
  pointer-events: none;
}
</style>
