<script setup lang="ts">
import { markRaw, onMounted, ref } from "vue";
import { loadRemote } from "./lib/loadremote";

const remote = ref(null);

onMounted(async () => {
  const remote = await loadRemote(
    "http://127.0.0.1:1234/static/TestRemote.vue"
  );
  remote.value = markRaw(remote);
  console.log(remote.value);
});
</script>

<template>
  <a href="#">hello world</a>
  <component v-if="remote" :is="remote"></component>
</template>

<style scoped></style>
