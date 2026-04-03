import { leftValue, meta } from "./root";
import { rightValue } from "./right";

it("should keep sibling reexport waves stable", () => {
  expect(leftValue).toBe("left");
  expect(rightValue).toBe("right");
  expect(meta.leftValueUsed).toBe(true);
  expect(meta.rightValueUsed).toBe(false);
});
