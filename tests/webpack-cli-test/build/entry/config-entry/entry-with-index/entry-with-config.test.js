"use strict";

const { run } = require("../../../../utils/test-utils");

describe("default entry and config entry all exist", () => {
  it("should use config entry if config entry existed", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // Should contain the relevant entry
    expect(stdout).toContain("./src/app.js");
    expect(stdout).toContain("./src/print.js");

    // Should contain the relevant bundle
    expect(stdout).toContain("app.bundle.js");
    expect(stdout).toContain("print.bundle.js");
    expect(stdout).not.toContain("index.js");
  });
});
