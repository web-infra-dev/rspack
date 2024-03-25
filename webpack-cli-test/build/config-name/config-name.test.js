"use strict";

const { run } = require("../../utils/test-utils");

describe("--config-name flag", () => {
  it("should select only the config whose name is passed with --config-name", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--config-name", "first"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("first");
    expect(stdout).not.toContain("second");
    expect(stdout).not.toContain("third");
  });

  it("should work with multiple values for --config-name", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config-name",
      "first",
      "--config-name",
      "third",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("first");
    expect(stdout).not.toContain("second");
    expect(stdout).toContain("third");
  });

  it("should work with multiple values for --config-name and multiple configurations", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      [
        "-c",
        "./function-config.js",
        "-c",
        "./single-other-config.js",
        "--config-name",
        "first",
        "--config-name",
        "four",
      ],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("first");
    expect(stdout).not.toContain("second");
    expect(stdout).not.toContain("third");
    expect(stdout).toContain("four");
  });

  it("should log error if invalid config name is provided", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--config-name", "test"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain('Configuration with the name "test" was not found.');
    expect(stdout).toBeFalsy();
  });

  it("should log error if multiple configurations are not found", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config-name",
      "test",
      "-c",
      "single-config.js",
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain('Configuration with the name "test" was not found.');
    expect(stdout).toBeFalsy();
  });

  it("should log error if multiple configurations are not found #1", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["--config-name", "test", "--config-name", "bar", "-c", "single-config.js"],
      false,
    );

    expect(exitCode).toBe(2);
    expect(stderr).toContain('Configuration with the name "test" was not found.');
    expect(stderr).toContain('Configuration with the name "bar" was not found.');
    expect(stdout).toBeFalsy();
  });

  it("should log error if multiple configurations are not found #2", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["--config-name", "first", "--config-name", "bar", "-c", "single-config.js"],
      false,
    );

    expect(exitCode).toBe(2);
    expect(stderr).toContain('Configuration with the name "bar" was not found.');
    expect(stdout).toBeFalsy();
  });

  it("should work with config as a function", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "function-config.js",
      "--config-name",
      "first",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("first");
    expect(stdout).not.toContain("second");
    expect(stdout).not.toContain("third");
  });

  it("should work with multiple values for --config-name when the config is a function", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["--config", "function-config.js", "--config-name", "first", "--config-name", "third"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("first");
    expect(stdout).not.toContain("second");
    expect(stdout).toContain("third");
  });

  it("should log error if invalid config name is provided ", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "function-config.js",
      "--config-name",
      "test",
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain('Configuration with the name "test" was not found.');
    expect(stdout).toBeFalsy();
  });
});
