import { Out } from "./lib/a"

it("should generate correct export for dynamic reexports (dynamic cjs)", () => {
  expect(Out).toBe(42)
})

