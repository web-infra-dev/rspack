"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe("default entry and config entry all exist", () => {
  it("should use config entry if config entry existed", async () => {
    const { stdout, stderr, exitCode } = await run(__dirname, ["-c", "../1.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("./a.js");
    expect(existsSync(resolve(__dirname, "./binary/index.bundle.js"))).toBeTruthy();
  });
});
