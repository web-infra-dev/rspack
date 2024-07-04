// eslint-disable-next-line node/no-unpublished-require
const { run } = require("../../../utils/test-utils");
const { existsSync } = require("fs");
const { resolve } = require("path");

describe("webpack cli", () => {
  it("should support typescript esnext file", async () => {
    const env = { ...process.env };

    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.config.mts"], {
      nodeOptions: ["--experimental-loader=ts-node/esm"],
      env,
    });

    expect(stderr).not.toBeFalsy(); // Deprecation warning logs on stderr
    expect(stdout).toBeTruthy();
    expect(exitCode).toBe(0);
    expect(existsSync(resolve(__dirname, "dist/foo.bundle.js"))).toBeTruthy();
  });
});
