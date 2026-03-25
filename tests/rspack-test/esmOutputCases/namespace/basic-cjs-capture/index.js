import * as ns from "./foo.js";

const value = 234;

it("should keep a captured CommonJS namespace object", () => {
  expect(ns).toHaveProperty("foo");
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
