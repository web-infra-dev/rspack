import { leftValue, leftValueUsed, rightValueUsed } from "./root";

it("should keep sibling reexport waves stable", async () => {
  const { rightValue } = await import("./root");

  expect(leftValue).toBe("left");
  expect(rightValue).toBe("right");
  expect(leftValueUsed).toBe(true);
  expect(rightValueUsed).toBe(false);
});
