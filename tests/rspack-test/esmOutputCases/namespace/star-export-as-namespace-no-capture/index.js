import { ns } from "./bar.js";

const value = 234;

it("should consume a re-exported namespace without capturing the object", () => {
  expect(ns.foo).toBe(123);
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
