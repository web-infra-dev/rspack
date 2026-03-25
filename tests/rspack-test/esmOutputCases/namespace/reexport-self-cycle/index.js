export { other } from "./other.js";

it("should support modules that re-export their own namespace", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.other.foo).toBe(1);
  expect(mod.other.bar).toBe(2);
  expect(mod.other.other.foo).toBe(1);
});
