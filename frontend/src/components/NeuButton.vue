<template>
  <button class="neu-btn" :class="variantClass" @mousedown="press" @mouseup="release" @mouseleave="release">
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'primary' | 'danger' | 'default'
}>(), { variant: 'default' })

const isPressed = ref(false)

const variantClass = computed(() => ({
  'neu-btn--primary': props.variant === 'primary',
  'neu-btn--danger': props.variant === 'danger',
  'neu-btn--pressed': isPressed.value,
}))

function press() { isPressed.value = true }
function release() { isPressed.value = false }
</script>

<style scoped>
.neu-btn {
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 12px 28px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  color: var(--neu-text);
  background: var(--neu-bg);
  box-shadow:
    5px 5px 10px var(--neu-shadow-dark),
    -5px -5px 10px var(--neu-shadow-light);
  transition: box-shadow var(--neu-transition), transform var(--neu-transition);
}

.neu-btn:hover {
  box-shadow:
    7px 7px 14px var(--neu-shadow-dark),
    -7px -7px 14px var(--neu-shadow-light);
  transform: translateY(-1px);
}

.neu-btn--pressed {
  box-shadow:
    inset 3px 3px 6px var(--neu-shadow-dark),
    inset -3px -3px 6px var(--neu-shadow-light) !important;
  transform: translateY(0) !important;
}

.neu-btn--primary {
  color: #fff;
  background: linear-gradient(145deg, var(--neu-primary-light), var(--neu-primary));
  box-shadow:
    5px 5px 10px var(--neu-shadow-dark),
    -5px -5px 10px rgba(91, 127, 255, 0.3);
}

.neu-btn--danger {
  color: #fff;
  background: var(--neu-danger);
}
</style>
