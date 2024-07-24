"use strict";

const { run } = require("../../../utils/test-utils");

// 'normal' is used in webpack.config.js
const statsPresets = [
  "detailed",
  "errors-only",
  "errors-warnings",
  "minimal",
  "verbose",
  "none",
  "summary",
];

describe("stats flag with config", () => {
  it("should compile without stats flag", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("preset: 'normal'");
  });

  for (const preset of statsPresets) {
    it(`should override 'normal' value in config with "${preset}"`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--stats", `${preset}`]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`preset: '${preset}'`);
    });
  }
});
