<script setup lang="ts">
import { computed } from "vue"

const props = defineProps<{
  code: string
  size?: string | number
}>()

const size = computed(() => `${props.size ?? 3}rem`)
</script>

<template>
  <span :style="{ fontSize: size }" class="material-icons">{{ props.code }}</span>
</template>
