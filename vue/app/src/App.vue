<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NavItem } from "./types";
import NaviVue from "./components/NaviVue.vue";
import Setting from "./components/SettingVue.vue";
import WindowHeaderVue from "./components/WindowHeaderVue.vue";
import type WujieVue from "wujie-vue3";
import LogoVue from "./components/LogoVue.vue";
import { commands } from "./generate/bridge";

const items = ref<NavItem[]>([]);
const version = ref("0.1.0");
const page = ref("");

async function loadItems(): Promise<NavItem[]> {
  return new Array(3).fill(new NavItem("Home1", 1));
}

onMounted(async () => (items.value = await loadItems()));

async function call() {
  const plugis = await commands.list_plugins();
  console.log(plugis);
}
</script>

<template>
  <div class="flex">
    <aside
      class="w-navi bg-navi flex h-screen flex-col border-r border-gray-200 shadow-md"
    >
      <header>
        <LogoVue />
      </header>
      <nav class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
        <NaviVue :items="items" />
      </nav>
      <footer>
        <Setting :version="version" />
      </footer>
    </aside>
    <main class="bg-page flex flex-1 flex-col">
      <header class="h-header" data-decoration>
        <WindowHeaderVue />
      </header>
      <article>
        <button @click="call">123123</button>
        <WujieVue class="flex h-full w-full" v-if="page" :url="page" />
      </article>
    </main>
  </div>
</template>
