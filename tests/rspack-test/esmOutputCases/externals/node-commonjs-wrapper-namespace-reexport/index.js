export * as exportsNS from "./exports";
export * from "./exports";

it("should keep a node-commonjs external wrapper on the require path only", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const { createRequire } = await import("node:module");
  const require = createRequire(import.meta.url);
  const sources = require("webpack-sources/lib/index.js");

  expect(mod.sources.RawSource).toBe(sources.RawSource);
  expect(mod.exportsNS.sources.RawSource).toBe(sources.RawSource);
  expect(mod.exportsNS.sources.RawSource).toBe(mod.sources.RawSource);
});
