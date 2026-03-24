export * as ns from "./foo.js";

it("should export a CommonJS namespace via export-star-as", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.ns.foo).toBe(123);
});
