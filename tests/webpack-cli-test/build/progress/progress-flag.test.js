"use strict";

const { run } = require("../../utils/test-utils");

describe("progress flag", () => {
  it("should show progress", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--progress"]);

    expect(exitCode).toBe(0);
    expect(stderr).not.toMatch(/\[webpack\.Progress] \d+ ms setup/);
    expect(stderr).toContain("[webpack.Progress] 100%");
    expect(stdout).toContain("main.js");
  });

  it('should support the "profile" value', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--progress=profile"]);

    expect(exitCode).toBe(0);
    expect(stderr).toMatch(/\[webpack\.Progress] \d+ ms setup/);
    expect(stderr).toContain("[webpack.Progress] 100%");
    expect(stdout).toContain("main.js");
  });

  it("should not support invalid value", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--progress=unknown"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(
      `'unknown' is an invalid value for the --progress option. Only 'profile' is allowed.`,
    );
    expect(stdout).toBeFalsy();
  });

  it("should not add duplicate plugins", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "webpack.progress.config.js",
      "--progress",
    ]);

    expect(exitCode).toEqual(0);
    expect(stderr).not.toMatch(/\[webpack\.Progress] \d+ ms setup/);
    expect(stderr).toContain("[webpack.Progress] 100%");
    expect(stdout).toContain("main.js");
    expect(stdout.match(/ProgressPlugin/g)).toHaveLength(1);
  });
});
