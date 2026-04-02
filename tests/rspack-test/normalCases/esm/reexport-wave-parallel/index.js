import { leftValue, rightValue } from "./root";

it("should keep sibling reexport waves stable", () => {
  expect(leftValue).toBe("left");
  expect(rightValue).toBe("right");
});
