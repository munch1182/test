<script setup lang="ts">
import { onMounted, ref } from "vue";
import NaviVue from "./components/NaviVue.vue";
import Setting from "./components/SettingVue.vue";
import WindowHeaderVue from "./components/WindowHeaderVue.vue";
import WujieVue from "wujie-vue3";
import LogoVue from "./components/LogoVue.vue";
import { commands } from "./generate/bridge";
import type { Plugin } from "@bridge/bridge";
import createPageState from "./utils/useAsync";
import Loadingbar from "./components/Loadingbar.vue";
import EmptyState from "./components/EmptyState.vue";

const items = ref<Plugin[]>([]);
const version = ref("0.1.0");
const curr = ref<Plugin | undefined>();
const state = createPageState();

onMounted(scan);

const scanParam = {
  p: {
    path: "../plugins/.dir",
    load_exists: false,
  },
};

async function scan() {
  await state.useAsync(() => commands.scan_plugins(scanParam));
  items.value = (await state.useAsync(() => commands.list_plugins())) ?? [];
  if (curr.value === undefined && items.value.length > 0) {
    curr.value = items.value[0];
  }
}

async function showSetting() {}

async function select(id: string) {
  console.log("select", id);
  curr.value = items.value.find((item) => item.id === id);
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
        <NaviVue :items="items" @select="select" :active-id="curr?.id ?? ''" />
      </nav>
      <footer>
        <Setting :version="version" @setting="showSetting" />
      </footer>
    </aside>
    <main class="bg-page relative flex flex-1 flex-col">
      <header class="h-header" data-decoration>
        <WindowHeaderVue />
      </header>
      <Loadingbar
        v-if="state.isLoading"
        class="top-header pointer-events-none absolute right-0 left-0 z-10"
      />
      <article class="flex-1 overflow-x-hidden overflow-y-auto">
        <EmptyState
          v-if="items.length === 0 || (curr?.url?.length ?? 0) <= 0"
          class="p-32"
        />
        <WujieVue class="flex h-full w-full" v-else :url="curr?.url ?? ''" />
      </article>
    </main>
  </div>
</template>
