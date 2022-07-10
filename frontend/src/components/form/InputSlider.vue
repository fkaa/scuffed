<script setup lang="ts">
import { computed, ref, useAttrs } from "vue"

interface Props {
  min?: number
  max?: number
  value: number
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  value: 0
})

// const attrs = useAttrs()

const emit = defineEmits<{
  (e: "update:value", value: number): void
}>()

const value = computed({
  get: () => props.value,
  set: (val) => emit("update:value", Number(val))
})
</script>

<template>
  <div class="input-slider">
    <input v-model="value" type="range" :min="props.min" :max="props.max" />
  </div>
</template>
