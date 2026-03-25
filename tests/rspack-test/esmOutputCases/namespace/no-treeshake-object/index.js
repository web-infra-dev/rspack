import * as ns1 from "./namespace.js";

export { ns1 };
export * as ns2 from "./namespace.js";
export const a = 1;
export const b = 2;

it("should keep namespace object exports when tree shaking is disabled", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.ns1.foo()).toBe("foo");
  expect(mod.ns2.foo()).toBe("foo");
  expect(Object.keys(mod.ns1)).toEqual(["foo"]);
  expect(mod.a).toBe(1);
  expect(mod.b).toBe(2);
});
