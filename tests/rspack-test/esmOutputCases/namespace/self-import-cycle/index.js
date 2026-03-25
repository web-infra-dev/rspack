import * as self from "./index.js";

const keys = Object.keys(self);

export { keys };
export var p = 5;

it("should support self namespace imports", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.p).toBe(5);
  expect(mod.keys).toContain("keys");
  expect(mod.keys).toContain("p");
});
