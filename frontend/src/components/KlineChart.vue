<template>
  <div class="chart-container">
    <canvas ref="canvasRef" class="chart-canvas"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'

interface Props {
  data: Array<{ time: number; open: number; high: number; low: number; close: number }>
}

const props = defineProps<Props>()
const canvasRef = ref<HTMLCanvasElement>()

function draw() {
  const canvas = canvasRef.value
  if (!canvas || !props.data.length) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.scale(dpr, dpr)

  const w = rect.width
  const h = rect.height
  const padding = { top: 20, right: 20, bottom: 40, left: 60 }
  const pw = w - padding.left - padding.right
  const ph = h - padding.top - padding.bottom

  const prices = props.data.flatMap((d) => [d.high, d.low])
  const min = Math.min(...prices)
  const max = Math.max(...prices)
  const range = max - min || 1

  const scaleX = (i: number) => padding.left + (i / (props.data.length - 1)) * pw
  const scaleY = (v: number) => padding.top + ((max - v) / range) * ph

  ctx.clearRect(0, 0, w, h)

  // Grid lines
  ctx.strokeStyle = '#e0e4e9'
  ctx.lineWidth = 0.5
  for (let i = 0; i <= 4; i++) {
    const y = padding.top + (i / 4) * ph
    ctx.beginPath()
    ctx.moveTo(padding.left, y)
    ctx.lineTo(w - padding.right, y)
    ctx.stroke()

    const price = max - (i / 4) * range
    ctx.fillStyle = '#8e8e9a'
    ctx.font = '11px Inter, sans-serif'
    ctx.fillText(price.toFixed(2), 4, y + 4)
  }

  // Candles
  const candleW = Math.max(2, pw / props.data.length * 0.7)
  props.data.forEach((d, i) => {
    const x = scaleX(i)
    const yOpen = scaleY(d.open)
    const yClose = scaleY(d.close)
    const yHigh = scaleY(d.high)
    const yLow = scaleY(d.low)

    const isGreen = d.close >= d.open
    ctx.fillStyle = isGreen ? '#4cd964' : '#ff5b5b'
    ctx.strokeStyle = isGreen ? '#4cd964' : '#ff5b5b'
    ctx.lineWidth = 1

    // Wick
    ctx.beginPath()
    ctx.moveTo(x, yHigh)
    ctx.lineTo(x, yLow)
    ctx.stroke()

    // Body
    const bodyH = Math.max(1, Math.abs(yClose - yOpen))
    const bodyY = Math.min(yOpen, yClose)
    ctx.fillRect(x - candleW / 2, bodyY, candleW, bodyH)
  })
}

onMounted(draw)
watch(() => props.data, draw)
</script>

<style scoped>
.chart-container {
  width: 100%;
  aspect-ratio: 2;
  border-radius: var(--neu-radius-sm);
  overflow: hidden;
}

.chart-canvas {
  width: 100%;
  height: 100%;
}
</style>
