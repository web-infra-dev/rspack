"use strict";

const { resolve } = require("path");
const { run } = require("../../../utils/test-utils");

describe("basic config file", () => {
  it("should build and not throw error with a basic configuration file", async () => {
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
