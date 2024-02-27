"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run, readdir } = require("../../../utils/test-utils");

describe("source-map object", () => {
  it("should not write a source map for obj config", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.eval.config.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("devtool: 'eval-cheap-module-source-map'");

    let files;

    try {
      files = await readdir(resolve(__dirname, "dist"));
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(files.length).toBeGreaterThanOrEqual(1);
  });

  it("should write a sourcemap file", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.source.config.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("devtool: 'source-map'");
    expect(existsSync(resolve(__dirname, "dist/dist-amd.js.map"))).toBeTruthy();
  });

  it("should override config with source-map", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["-c", "./webpack.eval.config.js", "--devtool", "source-map", "--output-path", "./binary"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("devtool: 'source-map'");
    expect(existsSync(resolve(__dirname, "binary/dist-amd.js.map"))).toBeTruthy();
  });

  it("should override config with devtool false", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["-c", "./webpack.eval.config.js", "--no-devtool", "--output-path", "./binary"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("devtool: false");
    expect(existsSync(resolve(__dirname, "binary/dist-amd.js.map"))).toBeTruthy();
  });
});
