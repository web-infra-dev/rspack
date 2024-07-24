"use strict";

const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../utils/test-utils");

describe("output flag defaults", () => {
  it("should create default file for a given directory", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--entry",
      "./a.js",
      "--output-path",
      "./binary",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // Should print warning about config fallback
    expect(stdout).toContain("option has not been set, webpack will fallback to");

    expect(existsSync(resolve(__dirname, "./binary/main.js"))).toBeTruthy();
  });

  it("set default output directory on no output flag", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--entry", "./a.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./binary/main.js"))).toBeTruthy();
  });

  it("throw error on empty output flag", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--entry",
      "./a.js",
      "--output-path",
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain("Error: Option '-o, --output-path <value>' argument missing");
    expect(stdout).toBeFalsy();
  });
});
