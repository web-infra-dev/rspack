export const foo = 123;

import * as ns from "./index.js";

export { ns };

it("should export its own namespace via import-and-export", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.foo).toBe(123);
  expect(mod.ns.foo).toBe(123);
});
