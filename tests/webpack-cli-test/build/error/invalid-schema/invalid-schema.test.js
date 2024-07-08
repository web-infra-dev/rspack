"use strict";
const { run } = require("../../../utils/test-utils");

describe("invalid schema", () => {
  it("should log error on invalid config", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "./webpack.mock.config.js",
    ]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid configuration object");
    expect(stdout).toBeFalsy();
  });

  it("should log error on invalid plugin options", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "./webpack.plugin-mock.config.js",
    ]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid options object");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid config using the "bundle" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "bundle",
      "--config",
      "./webpack.mock.config.js",
    ]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid configuration object");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid config using the "serve" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "serve",
      "--config",
      "./webpack.mock.config.js",
    ]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid configuration object");
    expect(stdout).toBeFalsy();
  });

  it("should log error on invalid option", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "build" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["build", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "bundle" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["bundle", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "b" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["b", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "watch" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["watch", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "w" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["w", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "server" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["serve", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });

  it('should log error on invalid option using "s" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["s", "--mode", "Yukihira"]);

    expect(exitCode).toEqual(2);
    expect(stderr).toContain("Invalid value 'Yukihira' for the '--mode' option");
    expect(stderr).toContain("Expected: 'development | production | none'");
    expect(stdout).toBeFalsy();
  });
});
