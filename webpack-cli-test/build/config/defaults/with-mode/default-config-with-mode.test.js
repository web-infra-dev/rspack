"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe("default config with mode from cli", () => {
  it("should build and not throw error with development mode supplied", async () => {
    const { stdout, stderr, exitCode } = await run(__dirname, ["--mode", "development"]);
    expect(exitCode).toEqual(0);
    expect(stderr).toBeFalsy();
    expect(stdout).not.toBe(undefined);
    expect(existsSync(resolve(__dirname, "./binary/dev.bundle.js"))).toBeTruthy();
  });
});
