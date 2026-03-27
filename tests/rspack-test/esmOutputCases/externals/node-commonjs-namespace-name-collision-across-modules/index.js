export { readFile, readFileSync } from "./named";
export { joinFn, pathNs } from "./path-ns";

it("should deconflict concatenated namespace imports that match generated node-commonjs names", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const { createRequire } = await import("node:module");
  const require = createRequire(import.meta.url);
  const fs = require("fs");
  const path = require("path");

  expect(mod.readFile).toBe(fs.readFile);
  expect(mod.readFileSync).toBe(fs.readFileSync);
  expect(mod.pathNs.sep).toBe(path.sep);
  expect(mod.pathNs.join).toBe(path.join);
  expect(mod.joinFn).toBe(path.join);
});
