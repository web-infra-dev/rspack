import * as ns from "./foo.js";

it("should render a missing CommonJS namespace property as undefined without capturing the namespace object", () => {
  expect(ns.x).toBe(123);
  expect(ns.foo).toBeUndefined();
});
