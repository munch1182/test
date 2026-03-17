<script setup lang="ts">
import { RiSubtractLine, RiCloseLine } from "@remixicon/vue";
// https://remixicon.com/
</script>

<template>
  <div
    class="flex h-full items-center justify-end border-b border-gray-200 bg-white px-4"
  >
    <button
      class="flex h-8 w-8 items-center justify-center rounded-full text-gray-600 hover:bg-gray-200"
      data-command="minimize"
    >
      <RiSubtractLine size="20" />
    </button>
    <button
      class="flex h-8 w-8 items-center justify-center rounded-full text-gray-600 hover:bg-red-100 hover:text-red-600"
      data-command="close"
    >
      <RiCloseLine size="20" />
    </button>
  </div>
</template>

<style scoped></style>
