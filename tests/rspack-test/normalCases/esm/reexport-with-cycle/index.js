import { f } from "./reexport"

it("should have correct value", () => {
  expect(f()).toBe(1);
})
