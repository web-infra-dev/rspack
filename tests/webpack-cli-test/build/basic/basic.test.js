"use strict";

const { resolve } = require("path");
const { run } = require("../../utils/test-utils");

describe("bundle command", () => {
  it("should work without command (default command)", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work without command and options (default command)", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--mode", "development"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work with multiple entries syntax without command (default command)", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["./src/index.js", "./src/other.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work with multiple entries syntax without command with options (default command)", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "./src/index.js",
      "./src/other.js",
      "--mode",
      "development",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work with multiple entries syntax without command with options #2 (default command)", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--mode",
      "development",
      "./src/index.js",
      "./src/other.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work with multiple entries syntax without command with options #3 (default command)", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "./src/index.js",
      "./src/other.js",
      "--entry",
      "./src/again.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work with and override entries from the configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "./src/index.js",
      "./src/other.js",
      "--config",
      "./entry.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with the "build" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["build"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with "bundle" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["bundle"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with the "b" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["b"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with entries syntax using the "build" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["build", "./src/index.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with entries syntax using the "bundle" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["bundle", "./src/index.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with entries syntax using the "b" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["b", "./src/index.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with multiple entries syntax using the "build" alias', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "build",
      "./src/index.js",
      "./src/other.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with multiple entries syntax using the "build" alias and options', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "build",
      "./src/index.js",
      "./src/other.js",
      "--mode",
      "development",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should work with multiple entries syntax using the "build" alias and options', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "build",
      "--mode",
      "development",
      "./src/index.js",
      "./src/other.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  // cSpell:ignore buil
  it('should log error and suggest right name on the "buil" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["buil"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain("Unknown command or entry 'buil'");
    expect(stderr).toContain("Did you mean 'build' (alias 'bundle, b')?");
    expect(stderr).toContain("Run 'webpack --help' to see available commands and options");
    expect(stdout).toBeFalsy();
  });

  it("should log supplied config when logging level is log", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--config", "./log.config.js"]);
    const configPath = resolve(__dirname, "./log.config.js");

    expect(exitCode).toBe(0);
    expect(stderr).toContain("Compiler starting...");
    expect(stderr).toContain(`Compiler is using config: '${configPath}'`);
    expect(stderr).toContain("Compiler finished");
    expect(stdout).toBeTruthy();
  });
});
