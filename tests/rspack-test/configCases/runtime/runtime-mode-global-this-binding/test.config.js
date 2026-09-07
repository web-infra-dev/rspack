const fs = require("fs");
const path = require("path");
const vm = require("vm");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
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
