import { ns } from "./foo.js";

it("should render a missing property from an exported namespace binding without capturing the object", () => {
  expect(ns.x).toBe(123);
  expect(ns.foo).toBeUndefined();
});
