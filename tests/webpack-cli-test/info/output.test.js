"use strict";

const { run } = require("../utils/test-utils");

describe("'-o, --output <value>' usage", () => {
  it("gets info text by default", async () => {
    const { exitCode, stdout, stderr } = await run(__dirname, ["info"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
  });

  it("gets info as json", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--output=json"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain('"System":');

    const parse = () => {
      const output = JSON.parse(stdout);
      expect(output["System"]).toBeTruthy();
      expect(output["Binaries"]).toBeTruthy();
      expect(output["System"]["OS"]).toBeTruthy();
      expect(output["System"]["CPU"]).toBeTruthy();
    };

    expect(parse).not.toThrow();
  });

  it("gets info as markdown", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--output", "markdown"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("## System:");
  });

  it("shows a warning if an invalid value is supplied", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--output", "unknown"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(`'unknown' is not a valid value for output`);
    expect(stdout).toBeFalsy();
  });

  it("recognizes '-o' as an alias for '--output'", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "-o", "markdown"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("## System:");
  });
});
