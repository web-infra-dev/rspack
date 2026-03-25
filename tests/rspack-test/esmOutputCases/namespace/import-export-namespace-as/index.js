import * as ns from "./foo.js";

export { ns };

it("should export an imported local namespace", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.ns.foo).toBe(123);
});
