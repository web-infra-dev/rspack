export * from "./foo.js";

it("should re-export a module that exports its own namespace", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.foo).toBe(123);
  expect(mod.ns.foo).toBe(123);
});
