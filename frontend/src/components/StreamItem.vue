<script setup lang="ts">
import { ref, reactive, computed } from "vue"
import { StreamItem } from "../bin/types"
import { getDurationSince } from "../bin/utils"

const props = defineProps<{
  data: StreamItem
}>()

const time = computed(() => getDurationSince(props.data.startedAt))
</script>

<template>
  <router-link class="stream-item" :to="{ name: 'Stream', params: { user: props.data.username } }">
    <div class="stream-thumbnail">
      <img :src="props.data.thumbnail" alt="" />
    </div>

    <div class="stream-info">
      <strong>{{ props.data.username }}</strong>

      <span>{{ props.data.views }} views</span>
      <div class="divider"></div>
      <span>Streaming for {{ time }} </span>
    </div>
  </router-link>
</template>
