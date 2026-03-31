import * as ns from "./bar.js";

const value = 234;

it("should keep a namespace imported from a star re-export when captured", () => {
  expect(ns).toHaveProperty("foo");
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
