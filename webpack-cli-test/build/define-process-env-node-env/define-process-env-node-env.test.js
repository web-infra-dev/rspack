"use strict";

const { run } = require("../../utils/test-utils");

describe("--define-process-env-node-env flag", () => {
  it('should set "process.env.NODE_ENV" to "development"', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--define-process-env-node-env",
      "development",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("mode: 'development'");
  });

  it('should set "process.env.NODE_ENV" to "production"', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--define-process-env-node-env",
      "production",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("mode: 'production'");
  });

  it('should set "process.env.NODE_ENV" to "none"', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--define-process-env-node-env",
      "none",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("mode: 'none'");
  });

  it('should set "process.env.NODE_ENV" and the "mode" option to "development"', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--define-process-env-node-env",
      "development",
      "--config",
      "./auto-mode.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("mode: 'development'");
  });

  it('should set "process.env.NODE_ENV" and the "mode" option to "production"', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--define-process-env-node-env",
      "production",
      "--config",
      "./auto-mode.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("mode: 'production'");
  });

  it('should set "process.env.NODE_ENV" and the "mode" option to "none"', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--define-process-env-node-env",
      "none",
      "--config",
      "./auto-mode.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("mode: 'none'");
  });
});
