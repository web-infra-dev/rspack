import * as nativeModule from "module";
const { createRequire: factory } = nativeModule;
const nativeRequire = factory(import.meta.url);
export function price() {
  return nativeRequire("./provider.cjs").price;
}
