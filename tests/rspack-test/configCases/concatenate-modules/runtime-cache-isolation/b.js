import { getValue } from "./root-shared";

it("should preserve the runtime-dependent export", () => {
  expect(getValue()).toBe(42);
});
