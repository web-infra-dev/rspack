"use strict";

const { join } = require("path");
const { run } = require("../utils/test-utils");

describe("'-a, --additional-package <value...>' usage", () => {
  it("should work with only one package", async () => {
    const { exitCode, stdout, stderr } = await run(join(__dirname, "../../"), [
      "version",
      "--additional-package",
      "typescript",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
    expect(stdout).toContain("typescript");
  });

  it("should work with only one package using '-a' alias", async () => {
    const { exitCode, stdout, stderr } = await run(join(__dirname, "../../"), [
      "version",
      "-a",
      "typescript",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
    expect(stdout).toContain("typescript");
  });

  it("should work with multiple packages", async () => {
    const { exitCode, stdout, stderr } = await run(join(__dirname, "../../"), [
      "version",
      "--additional-package",
      "typescript",
      "--additional-package",
      "eslint",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
    expect(stdout).toContain("typescript");
    expect(stdout).toContain("eslint");
    expect(stdout).toContain("eslint-config-prettier");
    expect(stdout).toContain("eslint-plugin-node");
  });

  it("should work with multiple packages using '-a' alias", async () => {
    const { exitCode, stdout, stderr } = await run(join(__dirname, "../../"), [
      "version",
      "-a",
      "typescript",
      "-a",
      "eslint",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
    expect(stdout).toContain("typescript");
    expect(stdout).toContain("eslint");
    expect(stdout).toContain("eslint-config-prettier");
    expect(stdout).toContain("eslint-plugin-node");
  });

  it("should throw an error on invalid usage", async () => {
    const { exitCode, stdout, stderr } = await run(join(__dirname, "../../"), [
      "version",
      "--additional-package",
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(`Option '-a, --additional-package <value...>' argument missing`);
    expect(stdout).toBeFalsy();
  });
});
