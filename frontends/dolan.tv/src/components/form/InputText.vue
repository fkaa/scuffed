<script setup lang="ts">
import { Error } from "../../bin/validation"

interface Props {
  label?: string
  value: string | number | undefined
  type?: string
  error?: Error
  required?: boolean
}

const { label, value, type = "text", error, required = false } = defineProps<Props>()
const emit = defineEmits<{
  (e: "update:value", value: string): void
}>()

function updateValue(e: any) {
  emit("update:value", e.target.value)
}
</script>

<template>
  <div class="form-input" :class="{ 'input-error': error && error.invalid, required: required }">
    <label v-if="label">{{ label }}</label>
    <input v-bind="$attrs" tabindex="0" class="border-smoke font-14" :type="type" @input="updateValue" :value="value" />
    <div class="input-error-list" v-if="error && error.invalid">
      <p v-for="item in error.errors">{{ item }}</p>
    </div>
  </div>
</template>
