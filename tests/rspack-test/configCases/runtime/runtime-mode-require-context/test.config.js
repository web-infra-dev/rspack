const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );

    expect(source).toContain("var __rspack_context={};");
    expect(source).toContain("__rspack_context.r");
    expect(source).toContain("module.exports, __rspack_context");
    expect(source).not.toContain("__rspack_context.d");
    expect(source).not.toContain("__rspack_context.N");
    expect(source).toMatch(/function __rspack_require\s*\(\s*moduleId\s*\)/);
    expect(source).toMatch(/var __rspack_modules\s*=/);
    expect(source).toMatch(/var __rspack_module_cache\s*=\s*\{\};/);
    expect(source).toContain("__rspack_context.r = __rspack_require;");
    expect(source).not.toMatch(/function __webpack_require__\s*\(/);
    expect(source).not.toMatch(/var __webpack_module_cache__\s*=/);
    expect(source).not.toMatch(/var __webpack_exports__\s*=/);
    expect(source).not.toMatch(/var __webpack_modules__\s*=/);
    expect(source).not.toContain(
      "// expose the modules object (__webpack_modules__)",
    );
    expect(source).not.toContain("__rspack_context.r = __webpack_require__;");
  },
};
