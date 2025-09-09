import { Out } from "./lib/a"

it("should generate correct export for dynamic reexports (consume shared module)", () => {
  expect(Out).toBe(42)
})
