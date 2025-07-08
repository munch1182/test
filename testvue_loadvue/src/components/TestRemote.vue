<script setup lang="ts">
import { ref } from "vue";

const index = ref(0);
</script>

<template>
  <div>{{ index }}</div>
  <input type="button" value="+" @click="index++" />
</template>
