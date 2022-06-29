<script setup lang="ts">
import { ref, reactive, computed } from "vue"
import { isEven } from "../../bin/utils"

const stream = ref(false)
</script>

<template>
  <div class="stream">
    <div class="stream-video">
      <video
        v-if="stream"
        src="https://www.youtube.com/watch?v=tP_Tz35iob4"
        controls
        class="video"
        id="video"
        preload="metadata"
        poster=""
      >
        <source src="https://www.youtube.com/watch?v=tP_Tz35iob4" />
      </video>

      <template v-else>
        <h2>dolanske is offline</h2>
        <img :src="`/images/${isEven(Date.now()) ? 'nostream2' : 'sleepy'}.png`" alt="" />

        <router-link :to="{ name: 'Streams' }" class="button">Other streams</router-link>
      </template>
    </div>
    <div class="stream-controls">

      <button class="">

      </button>
    </div>
  </div>
</template>
