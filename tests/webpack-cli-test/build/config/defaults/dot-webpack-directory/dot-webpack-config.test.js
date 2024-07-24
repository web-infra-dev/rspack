"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe(".webpack configuration file", () => {
  it("should build and not throw error when config is present in .webpack", async () => {
    const { stdout, stderr, exitCode } = await run(__dirname, []);
    expect(exitCode).toEqual(0);
    expect(stderr).toBeFalsy();
    expect(stdout).not.toBe(undefined);
    expect(existsSync(resolve(__dirname, "./binary/dev.bundle.js"))).toBeTruthy();
  });
});
