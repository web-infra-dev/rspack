"use strict";

const { resolve } = require("path");
const { run } = require("../../../utils/test-utils");

describe("custom config file", () => {
  it("should work with cjs format", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      resolve(__dirname, "config.webpack.js"),
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should work with esm format", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["--config", resolve(__dirname, "config.webpack.mjs")],
      {
        env: { WEBPACK_CLI_FORCE_LOAD_ESM_CONFIG: true },
      },
    );

    if (/Error: Not supported/.test(stderr)) {
      expect(exitCode).toBe(2);
      expect(stdout).toBeFalsy();
    } else {
      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
    }
  });
});
