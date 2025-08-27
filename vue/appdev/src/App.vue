<script setup lang="ts">
import { ref } from "vue";
import init, { pid } from "./wasm/wasm_appdev";

const result = ref(0);

init().then(() => {
  result.value = pid();
});
</script>

<template>
  <div class="container">{{ result }}</div>
</template>

<style scoped></style>
