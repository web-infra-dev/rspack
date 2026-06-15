const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const mainSource = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );
    const asyncChunkSource = fs
      .readdirSync(options.output.path)
      .filter(file => file.endsWith(".js") && file !== "main.js")
      .map(file =>
        fs.readFileSync(path.resolve(options.output.path, file), "utf-8"),
      )
      .join("\n");

    expect(mainSource).toContain("var __rspack_context={};");
    expect(mainSource).toContain("__rspack_context.r = __rspack_require;");
    expect(mainSource).toMatch(/function __rspack_require\s*\(\s*moduleId\s*\)/);
    expect(mainSource).toContain("__rspack_exports");
    expect(mainSource).toMatch(/var __rspack_module_cache\s*=\s*\{\};/);
    expect(mainSource).toContain(
      "// expose the modules object (__rspack_modules)",
    );
    expect(mainSource).not.toMatch(/function __webpack_require__\s*\(/);
    expect(mainSource).not.toMatch(/var __webpack_module_cache__\s*=/);
    expect(mainSource).not.toMatch(/var __webpack_exports__\s*=/);
    expect(mainSource).not.toMatch(/var __webpack_modules__\s*=/);
    expect(mainSource).not.toContain(
      "// expose the modules object (__webpack_modules__)",
    );
    expect(mainSource).not.toContain(
      "__rspack_context.r = __webpack_require__;",
    );
    expect(mainSource).toContain("module.exports, __rspack_context");
    expect(asyncChunkSource).toContain("__rspack_context.d");
    expect(asyncChunkSource).not.toContain("__rspack_install_runtime");
    expect(asyncChunkSource).not.toMatch(/function __webpack_require__\s*\(/);
    expect(asyncChunkSource).not.toMatch(/var __webpack_module_cache__\s*=/);
    expect(asyncChunkSource).not.toMatch(/var __webpack_modules__\s*=/);
    expect(asyncChunkSource).not.toContain(
      "// expose the modules object (__webpack_modules__)",
    );
  },
};
