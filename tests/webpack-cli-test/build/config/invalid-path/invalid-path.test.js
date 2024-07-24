"use strict";
const path = require("path");
const { run } = require("../../../utils/test-utils");

describe("config with invalid path supplied by CLI", () => {
  it("should throw error when invalid configuration path is passed to cli", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      path.resolve(__dirname, "invalid-webpack.config.js"),
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(
      `Failed to load '${path.resolve(__dirname, "invalid-webpack.config.js")}' config`,
    );
    expect(stdout).toBeFalsy();
  });
});
