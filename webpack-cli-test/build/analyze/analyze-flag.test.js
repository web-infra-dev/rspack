"use strict";

const { run, normalizeStdout } = require("../../utils/test-utils");

describe('"analyze" option', () => {
  it("should not load webpack-bundle-analyzer plugin twice with --analyze flag and plugin", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./analyze.config.js",
      "--analyze",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(normalizeStdout(stdout)).toContain("Webpack Bundle Analyzer saved report to");
    expect(normalizeStdout(stdout).match(/Webpack Bundle Analyzer saved report to/g)).toHaveLength(
      1,
    );
  });
});
