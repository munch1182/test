<script setup lang="ts">
import type { Plugin } from "@bridge/bridge";
const props = defineProps<{ items: Plugin[]; activeId?: string }>();
const emit = defineEmits<{ (e: 'select', id: string): void }>();
</script>

<template>
  <li v-for="item in items" :key="item.id" @click="emit('select', item.id)">
    <span :class="['pl-page block w-full py-3 transition', { 'bg-blue-500 text-white': activeId === item.id }]">
      {{ item.name }}
    </span>
  </li>
</template>

<style scoped></style>
