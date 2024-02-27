"use strict";

const { run, normalizeStderr, normalizeStdout } = require("../../../utils/test-utils");

const presets = [
  "normal",
  "detailed",
  "errors-only",
  "errors-warnings",
  "minimal",
  "verbose",
  "none",
  "summary",
];

describe("stats flag", () => {
  for (const preset of presets) {
    it(`should accept --stats "${preset}"`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--stats", `${preset}`]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`preset: '${preset}'`);
    });

    it("should accept stats as boolean", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--stats"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("preset: 'normal'");
    });
  }

  it("should accept stats as boolean", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--stats"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("preset: 'normal'");
  });

  it("should accept --no-stats as boolean", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--no-stats"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("preset: 'none'");
  });

  it("should log error when an unknown flag stats value is passed", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--stats", "foo"]);

    expect(exitCode).toEqual(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });
});
