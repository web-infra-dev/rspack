import * as fsNs from "fs";

export { fsNs };
export { readFile, readFileSync } from "fs";

it("should re-export an imported external namespace alongside named exports from the same source", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.fsNs.readFile).toBe(mod.readFile);
  expect(mod.fsNs.readFileSync).toBe(mod.readFileSync);
});
