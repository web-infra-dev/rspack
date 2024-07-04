"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe(".webpack webpackfile", () => {
  it("should build and not throw with .webpack webpackfile", async () => {
    const { stdout, stderr, exitCode } = await run(__dirname, []);
    expect(exitCode).toEqual(0);
    expect(stderr).toBeFalsy();
    expect(stdout).not.toBe(undefined);
    expect(existsSync(resolve(__dirname, "./binary/dev.folder.js"))).toBeTruthy();
  });
});
