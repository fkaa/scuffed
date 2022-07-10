<script setup lang="ts">
import "./style/index.scss"

import Navigation from "./components/Navigation.vue"
</script>

<template>
  <div class="app-layout">
    <Navigation />

    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in" appear>
        <component :is="Component"></component>
      </transition>
    </router-view>
  </div>
</template>
