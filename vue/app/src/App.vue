<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NavItem } from "./types";
import NaviVue from "./components/NaviVue.vue";
import Setting from "./components/SettingVue.vue";
import WindowHeaderVue from "./components/WindowHeaderVue.vue";
import type WujieVue from "wujie-vue3";
import LogoVue from "./components/LogoVue.vue";

const items = ref<NavItem[]>([]);
const version = ref("0.1.0");
const page = ref("");

async function loadItems(): Promise<NavItem[]> {
  return new Array(10).fill(new NavItem("Home1", 1));
}

onMounted(async () => (items.value = await loadItems()));
</script>

<template>
  <div class="flex">
    <aside
      class="w-navi h-screen bg-navi flex flex-col shadow-md border-r border-gray-200"
    >
      <header>
        <LogoVue />
      </header>
      <nav class="flex-1 overflow-y-auto overflow-x-hidden min-h-0">
        <NaviVue :items="items" />
      </nav>
      <footer>
        <Setting :version="version" />
      </footer>
    </aside>
    <main class="flex-1 bg-page flex flex-col">
      <header class="h-header">
        <WindowHeaderVue />
      </header>
      <article>
        <WujieVue class="w-full h-full flex" v-if="page" :url="page" />
      </article>
    </main>
  </div>
</template>
