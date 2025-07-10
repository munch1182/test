<script setup lang="ts">
import { loadModule } from "vue3-sfc-loader";
import * as Vue from "vue";
import { opts } from "./lib/loadremote";

const url = "http://127.0.0.1:1234/static/TestRemote.vue";

const remoteCompoment = Vue.defineAsyncComponent(async () =>
  loadModule(url, {
    moduleCache: { vue: Vue },
    ...opts,
  })
);
</script>

<template>
  <a href="#">hello world</a>
  <component :is="remoteCompoment" v-if="remoteCompoment" />
</template>

<style scoped></style>
