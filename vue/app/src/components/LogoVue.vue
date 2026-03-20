<script setup lang="ts">
import { RiCodeLine } from "@remixicon/vue";
import { onMounted, ref } from "vue";

const color = ref("#3B82F6");
const newColor = (): string => {
  const hex = Math.floor(Math.random() * 0xffffff).toString(16).padStart(6, '0');
  return `#${hex}`;
};
onMounted(() => color.value = newColor());
</script>

<template>
  <div class="flex flex-wrap items-center gap-4 p-4 text-xl font-bold text-gray-800">
    <RiCodeLine size="32" :color="color" />
    <span class="text-2xl">START</span>
  </div>
</template>

<style scoped></style>
