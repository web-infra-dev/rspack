"use strict";

const { run } = require("../../utils/test-utils");

describe("bundle variable", () => {
  it("compiles without flags and export variable", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("PASS");
  });
});
