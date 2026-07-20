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
    expect(source).toContain("__rspack_context.ctx = createContext;");
    expect(source).toContain("__rspack_context.ctx(map");
    expect(source).not.toContain("function __rspack_context_module_resolve(req)");
    expect(source).not.toContain("function __rspack_context(req)");
  },
};
