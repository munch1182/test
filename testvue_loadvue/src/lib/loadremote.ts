import * as Vue from "vue";
import { loadModule } from "vue3-sfc-loader";

export async function loadRemote(url: string) {
  const opts = {
    moduleCache: { vue: Vue },
    async getFile(url: string) {
      const res = await fetch(url);
      const code = await res.text();
      console.log(code);
      return code;
    },
    addStyle(textContent: string) {
      const style = Object.assign(document.createElement("style"), {
        textContent,
      });
      const ref = document.head.getElementsByTagName("style")[0] || null;
      document.head.insertBefore(style, ref);
    },
  };

  return Vue.defineAsyncComponent(async () => {
    return await loadModule(url, opts);
  });
}
