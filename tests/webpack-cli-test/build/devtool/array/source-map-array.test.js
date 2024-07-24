"use strict";

const { resolve } = require("path");
const { run, readdir } = require("../../../utils/test-utils");

describe("source-map object", () => {
  it("should treat source-map settings right", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // multi compilers
    expect(stdout).toContain("devtool: 'source-map'");
    expect(stdout).toContain("devtool: 'eval-cheap-module-source-map'");

    let files;

    try {
      files = await readdir(resolve(__dirname, "dist"));
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(files.length).toBe(3);
  });

  it("should override entire array on flag", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--devtool",
      "source-map",
      "--output-path",
      "./binary",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("devtool: 'source-map'");

    let files;

    try {
      files = await readdir(resolve(__dirname, "binary"));
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(files.length).toBe(4);
  });
});
