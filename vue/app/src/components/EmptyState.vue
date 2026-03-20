<script setup>
import { RiInboxLine } from "@remixicon/vue";

defineProps({
  message: { type: String, default: "暂无数据" },
});
</script>

<template>
  <div class="flex flex-col items-center justify-center text-gray-400">
    <RiInboxLine class="mb-4 h-12 w-12" />
    <p>{{ message }}</p>
  </div>
</template>
