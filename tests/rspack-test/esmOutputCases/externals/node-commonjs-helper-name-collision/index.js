import { createRequire as __rspack_createRequire } from "node:module";

const __rspack_createRequire_require = __rspack_createRequire(import.meta.url);
// fs
const external_fs_namespaceObject = __rspack_createRequire_require("fs");

export const sourceCreateRequireRequire = __rspack_createRequire_require;
export const sourceReadFile = external_fs_namespaceObject.readFile;
export const sourceRequireReadFile = require("fs").readFile;

it("should deconflict source bindings that already use the node-commonjs helper names", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const { createRequire } = await import(/* webpackIgnore: true */ "node:module");
  const require = createRequire(import.meta.url);
  const fs = require("fs");
  const path = require("path");
  const code = fs.readFileSync(path.join(__dirname, "main.mjs"), "utf-8");

  expect(mod.sourceReadFile).toBe(fs.readFile);
  expect(mod.sourceRequireReadFile).toBe(fs.readFile);
  expect(mod.sourceCreateRequireRequire("fs").readFile).toBe(fs.readFile);
  expect(code).toContain(
    'const __rspack_createRequire_require = __rspack_createRequire(import.meta.url);'
  );
  expect(code).toMatch(
    /^const (?!__rspack_createRequire_require\b)[A-Za-z0-9_$]*rspack_createRequire_require = .*createRequire.*;$/m
  );
});
