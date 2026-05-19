import * as ns from "./foo.js";

const value = 234;

it("should keep the namespace object when a direct import is captured", () => {
  expect(ns).toHaveProperty("foo");
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
