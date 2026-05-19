import { ns } from "./foo.js";

it("should keep an exported namespace binding when reading a missing property", () => {
  expect(ns).toHaveProperty("x");
  expect(ns.x).toBe(123);
  expect(ns.foo).toBeUndefined();
});
