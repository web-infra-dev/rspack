const { run } = require("../../../utils/test-utils");
const { existsSync } = require("fs");
const { resolve } = require("path");

describe("webpack cli", () => {
  it("should support typescript file", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.config.cts"]);

    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(exitCode).toBe(0);
    expect(existsSync(resolve(__dirname, "dist/foo.bundle.js"))).toBeTruthy();
  });
});
