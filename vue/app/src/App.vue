<script setup lang="ts">
import { onMounted, ref } from 'vue';
import NaviVue from './components/NaviVue.vue';
import { NavItem } from './types';
const items = ref<NavItem[]>([]);

async function loadItems(): Promise<NavItem[]> {
  return [
    new NavItem('Home1', 1),
    new NavItem('Home2', 2),
  ];
}

onMounted(async () => {
  items.value = await loadItems();
});

</script>
<template>
  <div class="flex min-h-screen">
    <nav class="w-navi bg-navi flex flex-col h-screen shadow-md">
      <header class="p-4 border-b border-gray-200">
        <div class="text-xl font-bold text-gray-800"> ⚡️ Logo</div>
      </header>

      <div class="flex-1 overflow-y-auto">
        <NaviVue :items="items" />
      </div>

      <footer class="p-4 border-t border-gray-200 flex items-center justify-between">
        <span class="text-gray-600 text-sm">v1.0.1</span>
        <button class="p-2 bg-gray-100 hover:bg-gray-200 rounded-full transition" title="设置">
          ⚙️
        </button>
      </footer>
    </nav>
  </div>
</template>
