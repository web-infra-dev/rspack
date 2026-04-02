import { leftValue, leftValueUsed, rightValueUsed } from "./root";

it("should keep sibling reexport waves stable", () => {
  expect(leftValue).toBe("left");
  expect(leftValueUsed).toBe(true);
  expect(rightValueUsed).toBe(false);
});
