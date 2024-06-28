"use strict";

const { resolve } = require("path");
const { run } = require("../../../utils/test-utils");

describe("top multi compiler options", () => {
  it("should work without provided configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("Done build0\nDone build1\nDone build2\nDone build3");
  });

  it("should work with provided configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      resolve(__dirname, "webpack.config.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("Done build0\nDone build1\nDone build2\nDone build3");
  });
});
