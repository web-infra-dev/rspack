"use strict";

const { run, readFile } = require("../../../utils/test-utils");
const { existsSync } = require("fs");
const { resolve } = require("path");

describe(" multiple entries", () => {
  it("should allow multiple entry flags", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--entry",
      "./src/a.js",
      "--entry",
      "./src/b.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/main.js"))).toBeTruthy();

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).toContain("Hello from b.js");
  });
});
