import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );

    expect(source).toContain("var __rspack_context={};");
    expect(source).toContain("function __rspack_context_module(req)");
    expect(source).not.toContain("function __rspack_context(req)");
  },
};
