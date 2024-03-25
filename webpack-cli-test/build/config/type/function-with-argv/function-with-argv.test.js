"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe("function configuration", () => {
  it("is able to understand a configuration file as a function", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--mode", "development"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(stdout).toContain("WEBPACK_BUNDLE: true");
    expect(stdout).toContain("WEBPACK_BUILD: true");
    expect(stdout).toContain("mode: 'development'");
    expect(existsSync(resolve(__dirname, "./dist/dev.js")));
  });
});
