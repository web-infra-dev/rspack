"use strict";

const { resolve } = require("path");
const { existsSync } = require("fs");
const { run } = require("../../../utils/test-utils");

describe("functional config", () => {
  it("should build and not throw error with single configuration", async () => {
    const { stderr, stdout, exitCode } = await run(__dirname, [
      "--config",
      resolve(__dirname, "single-webpack.config.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("./src/index.js");
    expect(existsSync(resolve(__dirname, "./dist/dist-single.js"))).toBeTruthy();
  });

  it("should build and not throw errors with multiple configurations", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      resolve(__dirname, "multi-webpack.config.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("first");
    expect(stdout).toContain("second");
    expect(existsSync(resolve(__dirname, "./dist/dist-first.js"))).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/dist-second.js"))).toBeTruthy();
  });
});
