export const foo = 123;
export * as ns from "./index.js";

it("should export its own namespace via export-star-as", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.foo).toBe(123);
  expect(mod.ns.foo).toBe(123);
});
