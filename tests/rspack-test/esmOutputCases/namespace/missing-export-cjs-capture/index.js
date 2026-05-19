import * as ns from "./foo.js";

it("should keep a CommonJS namespace object when reading a missing property", () => {
  expect(ns).toHaveProperty("x");
  expect(ns.x).toBe(123);
  expect(ns.foo).toBeUndefined();
});
