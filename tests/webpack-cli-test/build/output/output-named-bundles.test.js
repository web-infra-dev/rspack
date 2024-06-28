"use strict";

const { resolve } = require("path");
const { run, normalizeStdout, normalizeStderr } = require("../../utils/test-utils");

describe("output flag named bundles", () => {
  it("should output file given as flag instead of in configuration", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["-c", resolve(__dirname, "webpack.config.js"), "--output-path", "./binary"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should resolve the path to binary/a.bundle.js as ./binary/a.bundle.js", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["-c", resolve(__dirname, "webpack.config.js"), "--output-path", "binary"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should create multiple bundles with an overriding flag", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["-c", resolve(__dirname, "webpack.single.config.js"), "--output-path", "./bin"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should successfully compile multiple entries", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      resolve(__dirname, "webpack.multiple.config.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should output file in bin directory using default webpack config with warning for empty output value", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--output-path"]);

    expect(exitCode).toEqual(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });
});
