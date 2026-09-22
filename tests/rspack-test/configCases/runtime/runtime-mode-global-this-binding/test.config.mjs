import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  noTests: true,
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "bundle0.js"),
      "utf-8",
    );

    expect(source).toContain('"use strict";');
    expect(source).toContain("return this || new Function('return this')();");
    expect(source).toContain("\n}).call(this);\n");

    const workerGlobal = {
      Function: function BlockedFunction() {
        throw new Error("Blocked by CSP");
      },
      globalThis: undefined,
      importScripts() {},
      location: "https://example.test/worker.js",
    };
    workerGlobal.self = workerGlobal;

    expect(() => vm.runInNewContext(source, workerGlobal)).not.toThrow();
  },
};
