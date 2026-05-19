import { ns } from "./bar.js";

const value = 234;

it("should consume an export-star-as namespace binding without capturing the object", () => {
  expect(ns.foo).toBe(123);
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
