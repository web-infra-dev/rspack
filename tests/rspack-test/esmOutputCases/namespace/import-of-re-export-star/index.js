import * as ns from "./bar.js";

const value = 234;

it("should consume a namespace from a star re-export without capturing the object", () => {
  expect(ns.foo).toBe(123);
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
