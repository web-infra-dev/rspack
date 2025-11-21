import { Out } from "./lib/a"

it("should generate correct export for dynamic reexports (dynamic cjs)", () => {
  expect(Out).toBe(42)
})

it("should correctly concat modules", () => {
  const chunk = __STATS__.chunks[0];
  expect(chunk.names).toEqual(["main"]);
  expect(chunk.modules.map(module => module.name)).toEqual(["./index.js + 2 modules", "./lib/c.js"]);
})
