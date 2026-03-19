<script setup lang="ts">
import type { Plugin } from "@bridge/bridge";
const props = defineProps<{ items: Plugin[] }>();

async function select(navi: Plugin) {
  window.bridge.send("select", { id: navi.id });
}
</script>

<template>
  <ul
    class="w-navi bg-navi flex-1 list-none overflow-x-hidden overflow-y-auto py-2"
  >
    <li
      v-for="navi in props.items"
      :key="navi.id"
      @click.prevent="select(navi)"
    >
      <span
        class="pl-page block w-full py-3 transition hover:bg-gray-200"
        active-class="bg-blue-500 text-white"
      >
        {{ navi.name }}
      </span>
    </li>
  </ul>
</template>

<style scoped></style>
