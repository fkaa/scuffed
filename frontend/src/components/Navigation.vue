<script setup lang="ts">
import { ref, watch } from "vue"
import { useRoute } from "vue-router"

const route = useRoute()
const open = ref(false)

watch(open, (val) => {
  if (val) {
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") open.value = false
    })
  } else {
    window.removeEventListener("click", () => {})
  }
})

watch(
  () => route.name,
  () => (open.value = false)
)
</script>

<template>
  <div class="navigation" :class="{ active: open }" @click.self="open = false">
    <button
      class="nav-button"
      @click="open = !open"
      :class="{ active: open, 'has-shadow': route.name === 'Stream' }"
    ></button>

    <div class="nav-content">
      <router-link :to="{ name: 'Main' }">Home</router-link>
      <router-link :to="{ name: 'Account' }">Account</router-link>
      <router-link :to="{ name: 'Streams' }">Streams</router-link>
      <router-link :to="{ name: 'ViewDoc', params: { doc: 'intro' } }">Docs</router-link>
    </div>
  </div>
</template>
