import * as ns from "./bar.js";

const value = 234;

it("should tree-shake an unused namespace imported from a star re-export", () => {
  expect(value).toBe(234);
});
