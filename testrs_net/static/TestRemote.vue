<script setup lang="ts">
import { ref } from "vue";

const index = ref(0);
</script>

<template>
  <span>From remote .vue file</span>
  <div>{{ index }}11</div>
  <input type="button" value="+" @click="index++" />
</template>

<style scoped>
span {
  color: black;
  font-weight: bold;
}
</style>
