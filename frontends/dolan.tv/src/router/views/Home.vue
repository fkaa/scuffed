<script setup lang="ts">
//
</script>

<template>
  <div class="home">
    <div class="home-container">
      <div class="home-hero">
        <div class="logo">Scuffed.<b class="brand-gradient">tv</b></div>
        <p>Livestreaming service on a budget.</p>

        <router-link :to="{ name: 'Streams' }" class="button">Watch Streams</router-link>
      </div>

      <div class="home-graphic">
        <img src="/images/logo2.png" alt="" />
      </div>
    </div>
  </div>
</template>
