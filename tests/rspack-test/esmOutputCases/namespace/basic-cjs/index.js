import * as ns from "./foo.js";

it("should import a CommonJS namespace without capturing the object", () => {
  expect(ns.foo).toBe(123);
  expect(ns.foo).toBe(123);
});
