<script setup lang="ts">
import { ref } from "vue"
import { startStreamsUpdate } from "../../bin/stream"

import StreamItem from "../../components/StreamItem.vue"

const streams = ref()
startStreamsUpdate()
</script>

<template>
  <div class="streams">
    <div class="container">
      <template v-if="streams">
        <h1>All streams</h1>

        <div class="streams-list">
          <StreamItem v-for="stream in streams" :key="stream.username" :data="stream" />
        </div>
      </template>
      <template v-else>
        <h1>No streams</h1>
        <img class="no-stream" src="/images/nostream.png" alt="" />
      </template>
    </div>
  </div>
</template>
