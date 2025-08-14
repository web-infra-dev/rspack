const mod = require("./module");

it("should tree shake unused exports in shared modules", () => {
  expect(mod.used).toBe(42);
  expect(mod.unused).toBe(undefined);
});
