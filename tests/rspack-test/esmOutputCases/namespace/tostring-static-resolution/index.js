import * as foo from "./foo.js";

export const keep = foo.bar;

it("should keep namespace toStringTag visible for static resolution", () => {
  expect(Boolean(foo[Symbol.toStringTag])).toBe(true);
  expect(Boolean(foo.bar)).toBe(true);
  expect(Boolean(foo.foo)).toBe(false);
});
