import * as ns from "./foo.js";

it("should keep the namespace object for a module with an invalid named re-export", () => {
  expect(ns).toBeDefined();
  expect(ns.foo).toBeUndefined();
});
