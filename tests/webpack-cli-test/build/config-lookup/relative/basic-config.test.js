"use strict";

const { run } = require("../../../utils/test-utils");

describe("relative path to config", () => {
  it("should work", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "webpack.config.js",
      "--output-path",
      "./binary/a",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work #2", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./webpack.config.js",
      "--output-path",
      "./binary/b",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });
});
