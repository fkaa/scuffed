<script setup lang="ts">
import { ref, computed, watchEffect } from "vue";
import { useRoute } from "vue-router";
// @ts-ignore
import { marked } from "marked";

const route = useRoute();

const file = computed<string>(() => String(route.params.doc));
const content = ref();
const context = import.meta.glob("../../docs/*.md");

watchEffect(async () => {
  if (file) {
    const current = Object.keys(context).find((item) => item.includes(file.value));

    if (!current) return;

    const path = `${current}?raw`;
    const res = await import(/* @vite-ignore */ path);
    content.value = res.default;
  }
});
</script>

<template>
  <div v-if="content" class="content-style" v-html="marked.parse(content)"></div>
</template>
