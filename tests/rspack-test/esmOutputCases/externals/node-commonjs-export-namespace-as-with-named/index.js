export * as fsNs from "fs";
export { readFile, readFileSync } from "fs";

it("should combine node-commonjs export-star-as and named exports for the same source", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const { createRequire } = await import("node:module");
  const require = createRequire(import.meta.url);
  const fs = require("fs");

  expect(mod.fsNs.readFile).toBe(mod.readFile);
  expect(mod.fsNs.readFileSync).toBe(mod.readFileSync);
  expect(mod.fsNs.readFile).toBe(fs.readFile);
  expect(mod.fsNs.readFileSync).toBe(fs.readFileSync);
});
