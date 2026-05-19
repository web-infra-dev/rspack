import { ns } from "./bar.js";

const value = 234;

it("should tree-shake an unused re-exported namespace binding", () => {
  expect(value).toBe(234);
});
