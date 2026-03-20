<script setup>
import { RiInboxLine } from "@remixicon/vue";

defineProps({
  message: { type: String, default: "暂无数据" },
  icons: { type: Boolean, default: true },
});
</script>

<template>
  <div class="flex flex-col items-center justify-center text-gray-400">
    <RiInboxLine class="mb-4 h-12 w-12" v-if="icons" />
    <p>{{ message }}</p>
  </div>
</template>
