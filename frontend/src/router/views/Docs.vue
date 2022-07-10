<script setup lang="ts">
const nav = [
  {
    name: "",
    links: [
      {
        name: "Introduction",
        file: "intro"
      }
    ]
  },
  {
    name: "Streaming",
    links: [
      {
        name: "Optimize bandwidth",
        file: "optimize-bandwidth"
      },
      {
        name: "Using OBS",
        file: "stream-obs"
      },
      {
        name: "Stream visibility",
        file: "stream-visibility"
      }
    ]
  }
]
</script>

<template>
  <div class="docs">
    <div class="docs-nav">
      <div class="docs-nav-sticky-wrap">
        <div class="docs-nav-group" v-for="group in nav">
          <span v-if="group.name">{{ group.name }}</span>

          <router-link
            class="button btn-small"
            v-for="link in group.links"
            :to="{ name: 'ViewDoc', params: { doc: link.file } }"
          >
            {{ link.name }}
          </router-link>
        </div>
      </div>
    </div>
    <div class="docs-content">
      <router-view></router-view>
    </div>
  </div>
</template>
