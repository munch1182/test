<script setup lang="ts">
import { RiSettings2Fill } from "@remixicon/vue";
const prop = defineProps({ version: String });
</script>

<template>
  <div
    class="h-header flex items-center justify-between border-t border-gray-200 p-4"
  >
    <span class="text-sm text-gray-500">{{ prop.version }}</span>
    <button
      class="rounded-full p-2 transition hover:bg-gray-200"
      title="设置"
    >
      <RiSettings2Fill class="text-sm text-gray-500" />
    </button>
  </div>
</template>

<style scoped></style>
