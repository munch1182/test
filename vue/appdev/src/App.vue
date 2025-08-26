<script setup lang="ts">
import { ref, Suspense } from "vue";
import init, { add } from "./wasm/wasm_appdev";

const result = ref(0);

init().then(() => {
  result.value = add(2, 3);
});
</script>

<template>
  <div class="container">{{ result }}</div>
</template>

<style scoped></style>
