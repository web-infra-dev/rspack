"use strict";

const { resolve } = require("path");
const { run } = require("../../../utils/test-utils");

describe("config with single config in array", () => {
  it("should build and not throw error with configuration file exporting single configuration in array", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      resolve(__dirname, "webpack.config.js"),
      "--output-path",
      "./binary",
    ]);
    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });
});
