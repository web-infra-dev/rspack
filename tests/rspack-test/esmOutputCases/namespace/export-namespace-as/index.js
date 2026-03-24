export * as dep from "./dep.js";

it("should export a local namespace with direct export-star-as syntax", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.dep.foo).toBe("foo1");
  expect(mod.dep.bar).toBe("bar1");
});
