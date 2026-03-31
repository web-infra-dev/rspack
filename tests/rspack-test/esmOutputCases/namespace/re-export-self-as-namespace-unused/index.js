export { foo } from "./foo.js";

it("should not leak an unused self namespace re-export through the entry", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.foo).toBe(123);
  expect("ns" in mod).toBe(false);
});
