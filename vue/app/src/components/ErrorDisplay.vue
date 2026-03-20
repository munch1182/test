<script setup>
import { RiErrorWarningFill } from "@remixicon/vue";

defineProps({
  message: { type: String, default: "发生未知错误" },
  retry: { type: Function },
});
</script>

<template>
  <div
    class="flex flex-col items-center justify-center p-8 text-center text-red-500"
  >
    <RiErrorWarningFill class="mb-4 h-12 w-12" />
    <p class="mb-4">{{ message }}</p>
    <button
      v-if="retry"
      @click="retry"
      class="rounded-md bg-blue-500 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-600"
    >
      重试
    </button>
  </div>
</template>
