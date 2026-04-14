import * as util from "./util.js";

export const presentNull = util.foo?.nullVal?.x;
export const missingValue = util.x;
export const missingChain = util.x?.x;
export const namespaceChain = util?.x?.x;

it("should preserve optional chaining with namespace objects", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.presentNull).toBeUndefined();
  expect(mod.missingValue).toBeUndefined();
  expect(mod.missingChain).toBeUndefined();
  expect(mod.namespaceChain).toBeUndefined();
});
