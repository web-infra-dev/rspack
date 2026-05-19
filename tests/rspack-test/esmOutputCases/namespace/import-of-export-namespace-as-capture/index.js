import { ns } from "./bar.js";

const value = 234;

it("should keep an export-star-as namespace object when captured by the importer", () => {
  expect(ns).toHaveProperty("foo");
  expect(ns.foo).toBe(123);
  expect(value).toBe(234);
});
