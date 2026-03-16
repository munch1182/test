<script setup lang="ts">
import { RiSettings2Fill } from "@remixicon/vue";
const prop = defineProps({ version: String });
</script>

<template>
  <div
    class="h-header p-4 border-t border-gray-200 flex items-center justify-between"
  >
    <span class="text-gray-500 text-sm">{{ prop.version }}</span>
    <button
      class="bg-gray-100 hover:bg-gray-200 rounded-full transition"
      title="设置"
    >
      <RiSettings2Fill class="text-gray-500 text-sm" />
    </button>
  </div>
</template>

<style scoped></style>
