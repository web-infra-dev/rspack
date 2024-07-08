"use strict";

const { resolve } = require("path");
const { run } = require("../../../utils/test-utils");

describe("config with empty array", () => {
  it("should build and not throw error with empty configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      resolve(__dirname, "webpack.config.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeFalsy();
  });
});
