export { x as "'x" } from "./dep.js";

function a() {
  return "a";
}

function b() {
  return "b";
}

export { a as "'a", b as "'b" };

it("should support string export names that start with a quote", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod["'x"]).toBe(1);
  expect(mod["'a"]()).toBe("a");
  expect(mod["'b"]()).toBe("b");
});
