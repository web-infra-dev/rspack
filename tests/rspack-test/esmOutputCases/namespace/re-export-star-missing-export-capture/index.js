import * as ns from "./foo.js";

it("should keep the namespace object when reading a missing property through export-star", () => {
  expect(ns).toHaveProperty("x");
  expect(ns.x).toBe(123);
  expect(ns.foo).toBeUndefined();
});
