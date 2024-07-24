"use strict";
const { resolve } = require("path");
const { run } = require("../../../utils/test-utils");

describe("config with invalid export", () => {
  it("should throw error with configuration exporting invalid configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      resolve(__dirname, "webpack.config.js"),
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(
      `Invalid configuration in '${resolve(__dirname, "webpack.config.js")}'`,
    );
    expect(stdout).toBeFalsy();
  });
});
