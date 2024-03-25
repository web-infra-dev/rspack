"use strict";

const { run } = require("../../../utils/test-utils");

describe("stats flag", () => {
  it(`should use stats 'detailed' as defined in webpack config`, async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("preset: 'detailed'");
  });

  it(`should use --no-stats and override value in config`, async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--no-stats"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("preset: 'none'");
  });
});
