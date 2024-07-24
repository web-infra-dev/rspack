const fs = require("fs");
const path = require("path");
const { run } = require("../../../../utils/test-utils");

describe("default config with cjs extension", () => {
  it("should build and not throw error with cjs config by default", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toEqual(0);
    expect(stderr).toBeFalsy();
    // default entry should be used
    expect(stdout).toContain("./src/index.js");
    // should pick up the output path from config
    expect(stdout).toContain("test-output");
    expect(stdout).toContain("compiled successfully");
    // check that the output file exists
    expect(fs.existsSync(path.join(__dirname, "/dist/test-output.js"))).toBeTruthy();
  });
});
