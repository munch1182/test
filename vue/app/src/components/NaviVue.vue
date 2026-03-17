<script setup lang="ts">
import type { NavItem } from "../types";

const props = defineProps<{ items: NavItem[] }>();
</script>

<template>
  <ul
    class="w-navi bg-navi flex-1 list-none overflow-x-hidden overflow-y-auto py-2"
  >
    <li v-for="navi in props.items" :key="navi.id">
      <router-link
        :to="navi.url"
        class="pl-page block w-full py-3 transition hover:bg-gray-200"
        active-class="bg-blue-500 text-white"
      >
        {{ navi.name }}
      </router-link>
    </li>
  </ul>
</template>

<style scoped></style>
