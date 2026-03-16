<script setup lang="ts">
import type { NavItem } from '../types';

const props = defineProps<{ items: NavItem[] }>();
</script>

<template>
  <ul class="w-navi bg-navi py-2 list-none">
    <li v-for="navi in props.items" :key="navi.id">
      <router-link
        :to="navi.url"
        class="block pl-4 py-3 hover:bg-gray-200 transition"
        active-class="bg-blue-500 text-white"
      >
        {{ navi.name }}
      </router-link>
    </li>
  </ul>
</template>

<style scoped></style>