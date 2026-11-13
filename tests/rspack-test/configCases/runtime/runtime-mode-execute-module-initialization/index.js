import value from "./loader.mjs!./source";
it("shares build-time module cache and execution interceptors with runtime context", () => {
  expect(value).toEqual({ value: 42, cached: true, intercepted: 1, path: "from-module/" });
});
