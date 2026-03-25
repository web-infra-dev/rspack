import * as mod from "./mod.js";

export default mod.default();

it("should expose the default export through a namespace object", async () => {
  const out = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default()).toBe(42);
  expect(out.default).toBe(42);
});
