<script setup lang="ts">
import { onMounted, ref } from "vue";
import NaviVue from "./components/NaviVue.vue";
import Setting from "./components/SettingVue.vue";
import WindowHeaderVue from "./components/WindowHeaderVue.vue";
import type WujieVue from "wujie-vue3";
import LogoVue from "./components/LogoVue.vue";
import { commands } from "./generate/bridge";
import type { Plugin } from "@bridge/bridge";;

const items = ref<Plugin[]>([]);
const version = ref("0.1.0");
const page = ref("");

onMounted(async () => (items.value = await commands.list_plugins()));

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
        <WujieVue class="flex h-full w-full" v-if="page" :url="page" />
      </article>
    </main>
  </div>
</template>
