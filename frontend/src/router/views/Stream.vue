<script setup lang="ts">
import InputSlider from "../../components/form/InputSlider.vue"
import { ref, reactive, onMounted, watch } from "vue"
import { isEven } from "../../bin/utils"
import { onKeyStroke, useMagicKeys, whenever } from "@vueuse/core"
import { isNil } from "lodash"

const defaultVolume = Number(localStorage.getItem("stream-vol") ?? 30)
const defaultIncrement = 5
const stream = ref<HTMLMediaElement>()

onMounted(() => {
  const el = document.querySelector<HTMLMediaElement>("#video")

  if (el) {
    stream.value = el

    stream.value.volume = defaultVolume / 100
  }
})

// Video controls
const state = reactive({
  playing: false,
  UI: true,
  UIHover: false,
  volume: defaultVolume,
  UIVolume: false
})

const Play = () => (stream.value ? stream.value.play() : null)
const Pause = () => (stream.value ? stream.value.pause() : null)

// Mouse movement
let timeout: NodeJS.Timeout

document.addEventListener("mousemove", (e) => {
  clearTimeout(timeout)

  if (state.playing) {
    state.UI = true
    timeout = setTimeout(() => {
      if (!state.UIHover) {
        state.UI = false
        state.UIVolume = false
      }
    }, 2000)
  }
})

watch(
  () => state.playing,
  (val) => {
    if (val) {
    } else {
      state.UI = true
    }
  }
)

/**
 * Keyboard shortcuts & control
 */
const { Equal, Minus, Space } = useMagicKeys()

onKeyStroke(["k"], () => {
  if (state.playing) Pause()
  else Play()
})

// Volume

watch(
  () => state.volume,
  (value) => {
    state.UIVolume = true
    localStorage.setItem("stream-volume", String(value))

    if (stream.value) {
      stream.value.volume = value / 100
    }
  }
)

whenever(Space, () => {
  if (state.playing) Pause()
  else Play()
})

whenever(Equal, () => {
  state.volume = Math.min(100, state.volume + defaultIncrement)
})

whenever(Minus, () => {
  state.volume = Math.max(0, state.volume - defaultIncrement)
})
</script>

<template>
  <div class="stream">
    <router-link
      class="stream-button exit"
      :to="{ name: 'Streams' }"
      data-title-right="Back to stream list"
    >
      <Icon code="west" />
    </router-link>

    <div class="stream-video">
      <video
        v-if="true"
        class="video"
        id="video"
        preload="metadata"
        poster=""
        @pause="state.playing = false"
        @play="state.playing = true"
      >
        <source src="/test.mp4" type="video/mp4" />
      </video>

      <template v-else>
        <h2>dolanske is offline</h2>
        <img :src="`/images/${isEven(Date.now()) ? 'nostream2' : 'sleepy'}.png`" alt="" />
        <router-link :to="{ name: 'Streams' }" class="button">Other streams</router-link>
      </template>
    </div>
    <div
      class="stream-controls"
      v-if="stream"
      :class="{ active: state.UI }"
      @mouseenter="state.UIHover = true"
      @mouseleave="state.UIHover = false"
    >
      <button class="stream-button" v-show="!state.playing" @click="Play()" data-title-top="Play">
        <Icon code="play_arrow" />
      </button>
      <button class="stream-button" v-show="state.playing" @click="Pause()" data-title-top="Pause">
        <Icon code="pause" />
      </button>
      <button
        class="stream-button"
        :data-title-top="`Volume (${state.volume}%)`"
        @click="state.UIVolume = !state.UIVolume"
      >
        <Icon code="volume_up" />
      </button>

      <InputSlider v-if="state.UIVolume" v-model:value="state.volume" />

      <!-- <button class="stream-button" data-title-top="Mute"><Icon code="volume_off" /></button> -->
      <!-- <div class="flex-1"></div> -->
      <!-- <button class="stream-button" data-title-top="Open chat"><Icon code="chat" /></button> -->

      <!-- <div class="flex-1"></div> -->
    </div>
  </div>
</template>
