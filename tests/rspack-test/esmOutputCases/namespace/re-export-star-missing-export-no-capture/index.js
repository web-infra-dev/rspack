import * as ns from "./foo.js";

it("should render a missing namespace property through export-star without capturing the namespace object", () => {
  expect(ns.x).toBe(123);
  expect(ns.foo).toBeUndefined();
});
