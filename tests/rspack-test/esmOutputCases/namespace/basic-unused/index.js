import * as ns from "./foo.js";

const value = 234;

it("should tree-shake an unused direct namespace import", () => {
  expect(value).toBe(234);
});
