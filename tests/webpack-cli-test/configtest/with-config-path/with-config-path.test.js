"use strict";

const { run, normalizeStderr, normalizeStdout } = require("../../utils/test-utils");

describe("'configtest' command with the configuration path option", () => {
  it("should validate webpack config successfully", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["configtest", "./basic.config.js"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should throw validation error", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["configtest", "./error.config.js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should throw syntax error", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "configtest",
      "./syntax-error.config.js",
    ]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it(`should validate the config with alias 't'`, async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["t", "./error.config.js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should throw error if configuration does not exist", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["configtest", "./a.js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr).split("\n")[0]).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });
});
