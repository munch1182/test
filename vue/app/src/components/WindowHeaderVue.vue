<script setup lang="ts">
import { RiSubtractLine, RiCloseLine } from "@remixicon/vue";
// https://remixicon.com/
</script>

<template>
  <div
    class="h-full flex items-center justify-end px-4 border-b border-gray-200 bg-white"
  >
    <button
      class="w-8 h-8 flex items-center justify-center hover:bg-gray-200 rounded-full text-gray-600"
    >
      <RiSubtractLine size="20" />
    </button>
    <button
      class="w-8 h-8 flex items-center justify-center hover:bg-red-100 hover:text-red-600 rounded-full text-gray-600"
    >
      <RiCloseLine size="20" />
    </button>
  </div>
</template>

<style scoped></style>
