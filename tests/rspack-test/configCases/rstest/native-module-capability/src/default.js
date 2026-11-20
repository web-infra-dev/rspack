import nativeModule from "node:module";
const holder = { factory: nativeModule.createRequire };
const nativeRequire = holder.factory(import.meta.url);
export function price() {
  return nativeRequire("./provider.cjs").price;
}
