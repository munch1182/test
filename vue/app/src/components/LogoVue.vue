<script setup lang="ts">
import { RiCodeLine } from "@remixicon/vue";
import { onMounted, ref } from "vue";

const color = ref("#3B82F6");
function newColor(): string {
  return "#" + Math.floor(Math.random() * 16777215).toString(16);
}
onMounted(() => {
  color.value = newColor();
});
</script>

<template>
  <div
    class="text-xl font-bold flex flex-wrap gap-4 items-center text-gray-800 p-4"
  >
    <RiCodeLine size="32" :color="color" />
    <span class="text-2xl">START</span>
  </div>
</template>

<style scoped></style>
