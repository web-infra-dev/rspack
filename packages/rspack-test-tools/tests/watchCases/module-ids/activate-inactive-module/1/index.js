import { value } from "./a";

it("should not change the module id for the updated module", async () => {
  expect(globalThis["activate-inactive-module"]).toBe(42);
  expect(value).toBe(42);
})
