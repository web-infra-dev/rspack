import { Out } from "./lib/a"

it("should generate correct export for dynamic reexports (consume shared module)", () => {
  expect(Out).toBe(42)
})

it("should correctly concat modules", () => {
  const chunk = __STATS__.chunks[0];
  expect(chunk.names).toEqual(["main"]);
  expect(chunk.modules.map(module => {
    if (module.name.startsWith("consume shared module")) {
      return module.name.slice(0, 44)
    }
    return module.name
  })).toEqual(["./index.js + 2 modules", "./lib/c.js", "consume shared module (default) ./lib/c.js@*"]);
})
