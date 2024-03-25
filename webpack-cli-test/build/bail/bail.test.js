"use strict";

const { run } = require("../../utils/test-utils");

describe("bail and watch warning", () => {
  it("should not log warning in not watch mode", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "bail-webpack.config.js"]);

    expect(exitCode).toEqual(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it('should not log warning in not watch mode without the "bail" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "no-bail-webpack.config.js"]);

    expect(exitCode).toEqual(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });
});
