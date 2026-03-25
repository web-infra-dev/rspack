import * as fooNs from "./foo.js";

export { fooNs };

it("should import a module that exports another namespace", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.fooNs.barNs.bar).toBe(123);
});
