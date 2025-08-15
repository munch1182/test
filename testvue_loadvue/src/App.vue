<script setup lang="ts">
import { loadModule } from "vue3-sfc-loader";
import * as Vue from "vue";
import { opts } from "./lib/loadremote";
import WujieVue from "wujie-vue3";

const urlVue = "http://127.0.0.1:1234/static/TestRemote.vue";
const urlHtml = "http://127.0.0.1:1234/static/dist/index.html";

const remoteCompoment = Vue.defineAsyncComponent(async () =>
  loadModule(urlVue, {
    moduleCache: { vue: Vue },
    ...opts,
  })
);
</script>

<template>
  <div>
    <a href="#">hello world</a>
  </div>
  <hr>
  <div class="h-20">
    <div>使用sfc加载远程单vue文件:</div>
    <component :is="remoteCompoment" v-if="remoteCompoment" />
  </div>
  <hr>
  <div>
    <div>使用wujie加载远程网站:</div>
    <WujieVue :url="urlHtml" name="vue3"></WujieVue>
  </div>
</template>

<style scoped></style>
