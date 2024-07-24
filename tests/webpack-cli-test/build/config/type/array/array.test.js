"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe("array config", () => {
  it("is able to understand a configuration file in array format", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      resolve(__dirname, "webpack.config.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/dist-commonjs.js"))).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/dist-amd.js"))).toBeTruthy();
  });

  it("respect cli args with config as an array", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--stats", "none"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // should not print anything because of stats: none
    expect(stdout).toBeFalsy();
    expect(existsSync(resolve(__dirname, "./dist/dist-commonjs.js"))).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/dist-amd.js"))).toBeTruthy();
  });
});
