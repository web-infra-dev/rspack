const __rspack_createRequire_require = "user-defined";

export const userCreateRequireRequire = __rspack_createRequire_require;
export { readFile } from "fs";

it("should rename user bindings that collide with the node-commonjs helper", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const { createRequire } = await import(/* webpackIgnore: true */ "node:module");
  const require = createRequire(import.meta.url);
  const fs = require("fs");
  const outputPath = require("path");
  const code = fs.readFileSync(outputPath.join(__dirname, "main.mjs"), "utf-8");

  expect(mod.userCreateRequireRequire).toBe("user-defined");
  expect(mod.readFile).toBe(fs.readFile);
  expect(code).toContain(
    'const __rspack_createRequire_require = __rspack_createRequire(import.meta.url);'
  );
  expect(code).toMatch(
    /^const [A-Za-z0-9_$]*rspack_createRequire_require = "user-defined";$/m
  );
  expect(code).not.toMatch(/^const __rspack_createRequire_require = "user-defined";$/m);
});
