const fs = require("fs");
const path = require("path");

const source = fs.readFileSync(
  path.resolve(__dirname, "dist/main.js"),
  "utf-8",
);

expect(source).toContain("var __rspack_context={};");
expect(source).toContain("__rspack_context.r = __webpack_require__;");
expect(source).toContain("__rspack_context.esm");
expect(source).not.toContain("__rspack_context.d = definePropertyGetters;");
expect(source).not.toContain("__rspack_context.N = makeNamespaceObject;");
expect(source).toContain("__rspack_context.esm = defineEsmExports;");
expect(source).toContain("module.exports, __rspack_context");
expect(source).toContain("definePropertyGetters =");
expect(source).toContain("makeNamespaceObject =");
expect(source).toContain("defineEsmExports =");
expect(source).toContain("makeNamespaceObject(exports);");
expect(source).toContain("definePropertyGetters(exports, getters, values);");
expect(source.indexOf("definePropertyGetters(exports, getters, values);"))
  .toBeGreaterThan(source.indexOf("makeNamespaceObject(exports);"));
expect(source).toContain("__rspack_context.esm(__rspack_exports, {");
expect(source).not.toContain("__rspack_context.N(__rspack_exports);");
expect(source).not.toContain("__rspack_context.d(__rspack_exports, {");
expect(source).not.toContain("__webpack_require__.d(__webpack_exports__");
expect(source).not.toContain("__webpack_require__.r(__webpack_exports__");
expect(source).not.toContain("__webpack_require__.d =");
expect(source).not.toContain("__webpack_require__.r =");
