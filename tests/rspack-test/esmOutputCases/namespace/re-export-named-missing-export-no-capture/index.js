import * as ns from "./foo.js";

it("should render a missing property from a module with an invalid named re-export without capturing the namespace object", () => {
  expect(ns.foo).toBeUndefined();
});
