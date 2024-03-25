"use strict";
const { cli } = require("webpack");
const { run } = require("../test-utils");

describe("webpack-cli-test-plugin Test", () => {
  it("should log the webpack configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain(`target: 'node'`);

    if (typeof cli !== "undefined") {
      expect(stdout).toContain(`alias: { alias: [ 'alias1', 'alias2' ] }`);
    }

    expect(stdout).toContain("WebpackCLITestPlugin");
  });
});
