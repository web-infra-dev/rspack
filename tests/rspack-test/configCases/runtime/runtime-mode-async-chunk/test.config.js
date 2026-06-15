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
    expect(mainSource).toMatch(/var __rspack_module_cache\s*=\s*\{\};/);
    expect(mainSource).not.toContain("__webpack_require__");
    expect(mainSource).not.toContain("__webpack_module_cache__");
    expect(mainSource).toContain("module.exports, __rspack_context");
    expect(asyncChunkSource).toContain("__rspack_context.d");
    expect(asyncChunkSource).not.toContain("__rspack_install_runtime");
    expect(asyncChunkSource).not.toContain("__webpack_require__.d");
  },
};
