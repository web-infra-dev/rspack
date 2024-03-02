"use strict";

const { resolve } = require("path");
const { run } = require("../../utils/test-utils");

describe("custom-webpack", () => {
  it("should use package from 'node_modules'", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      env: { WEBPACK_PACKAGE: "webpack" },
    });

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("main.js");
  });

  it("should use custom-webpack.js", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      env: { WEBPACK_PACKAGE: resolve(__dirname, "./custom-webpack.js") },
    });

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("main.js");
  });
});
