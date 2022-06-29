<script setup lang="ts">
import { ref, reactive, computed } from "vue"
import { useClipboard, onClickOutside } from "@vueuse/core"

const show = ref(false)
const keywrap = ref(null)
const key = ref("853f86e8-e6b1-45d8-997f-aacabf153be6")

onClickOutside(keywrap, () => {
  show.value = false
})

// New key generation
const loading = ref(false)
const done = ref(false)

function generate() {
  loading.value = true

  setTimeout(() => {
    loading.value = false
    done.value = true

    setTimeout(() => {
      done.value = false
    }, 2000)
  }, 1500)
  console.log("damn generate")
}

// Copy
const { isSupported, copy, copied } = useClipboard()
</script>

<template>
  <div class="account">
    <h1>Account</h1>

    <div class="account-info">
      <div class="account-info-cell">
        <span>Username</span>
        <strong>dolanske</strong>
      </div>
      <div class="account-info-cell">
        <span>Email</span>
        <strong>dolanovsky@gmail.com</strong>
      </div>
    </div>

    <div class="buttons">
      <router-link :to="{ name: 'Password' }" class="button btn-small">Change Password</router-link>
    </div>

    <hr />

    <div class="stream-key">
      <h3>Stream key</h3>
      <p>Make sure to hide your key.</p>

      <div class="key" :class="{ 'is-open': show }" ref="keywrap">
        <input
          type="text"
          :class="{ 'no-select': !show }"
          :value="show ? key : '●●●●●●●●●●●●●●●●●●●●●●●●●●●●●●●●●●●'"
          readonly
          autocomplete="off"
        />

        <button data-title-bottom="Show stream key" @click="show = !show">
          <Icon v-if="show" code="visibility_off" size="2" />
          <Icon v-else code="visibility" size="2" />
        </button>

        <button data-title-bottom="Copy stream key" v-if="isSupported" @click="copy(key)">
          <Icon code="content_copy" size="2" />
          <span class="copied-label" :class="{ active: copied }">Copied!</span>
        </button>

        <button data-title-bottom="Generate new stream key" class="btn-generate" @click="generate">
          <Icon v-if="loading" class="rotate" code="refresh" size="2" />

          <Icon v-else code="refresh" size="2" />

          <span class="copied-label info" :class="{ active: done }">Completed!</span>
        </button>
      </div>
    </div>
  </div>
</template>
