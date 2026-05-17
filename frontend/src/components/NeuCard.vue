<template>
  <div class="neu-card" :class="{ 'neu-card--pressed': pressed }">
    <slot />
  </div>
</template>

<script setup lang="ts">
defineProps<{ pressed?: boolean }>()
</script>

<style scoped>
.neu-card {
  background: var(--neu-bg);
  border-radius: var(--neu-radius);
  padding: 24px;
  box-shadow:
    8px 8px 16px var(--neu-shadow-dark),
    -8px -8px 16px var(--neu-shadow-light);
  transition: box-shadow var(--neu-transition), transform var(--neu-transition);
}

.neu-card:hover {
  box-shadow:
    12px 12px 24px var(--neu-shadow-dark),
    -12px -12px 24px var(--neu-shadow-light);
  transform: translateY(-2px);
}

.neu-card--pressed {
  box-shadow:
    inset 4px 4px 8px var(--neu-shadow-dark),
    inset -4px -4px 8px var(--neu-shadow-light);
  transform: translateY(0);
}
</style>
