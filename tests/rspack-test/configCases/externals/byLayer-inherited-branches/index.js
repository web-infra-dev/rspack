import { value } from "./app";

it("should ignore inherited layer branches and defaults", () => {
  expect(value).toBe(42);
});
