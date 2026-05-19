import * as ns from "./reexport.js";

export default ns.foo;

it("should keep the first conflicting namespace export under a warning", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toBe(1);
});
