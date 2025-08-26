import { execSync } from "child_process";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

export default function buildWasm() {
  return {
    name: "build-wasm",
    buildStart() {
      console.log("start wasm build");
      try {
        const currentDir = dirname(fileURLToPath(import.meta.url));
        let rustdir = resolve(currentDir, "../../rust/wasm_appdev");
        let cmd = `cd ${rustdir} && wasm-pack build --target web --out-dir ${currentDir}/src/wasm`;
        execSync(cmd);

        console.log("wasm build success");
      } catch (e) {
        console.error(e);
      }
    },
  };
}
