import { a, b } from "./reexport";

it("should work", () => {
  expect(a).toBe(1);
  expect(b).toBe(2);
});
