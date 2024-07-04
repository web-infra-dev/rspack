"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run } = require("../../../../utils/test-utils");

describe("array of promises", () => {
  it("is able to understand a configuration file as a promise", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.config.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./binary/a-promise.js"))).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./binary/b-promise.js"))).toBeTruthy();
  });
});
