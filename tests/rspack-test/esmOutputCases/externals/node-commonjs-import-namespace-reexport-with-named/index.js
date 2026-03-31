import * as fsNs from "fs";

export { fsNs };
export { readFile, readFileSync } from "fs";

it("should re-export an imported node-commonjs namespace alongside named exports from the same source", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const { createRequire } = await import("node:module");
  const require = createRequire(import.meta.url);
  const fs = require("fs");

  expect(mod.fsNs.readFile).toBe(mod.readFile);
  expect(mod.fsNs.readFileSync).toBe(mod.readFileSync);
  expect(mod.fsNs.readFile).toBe(fs.readFile);
  expect(mod.fsNs.readFileSync).toBe(fs.readFileSync);
});
