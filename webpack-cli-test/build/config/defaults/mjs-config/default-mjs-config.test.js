const fs = require("fs");
const path = require("path");
const { run } = require("../../../../utils/test-utils");

describe("default config with mjs extension", () => {
  it("should build and not throw error with mjs config by default", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      env: { WEBPACK_CLI_FORCE_LOAD_ESM_CONFIG: true },
    });

    if (/Error: Not supported/.test(stderr)) {
      expect(exitCode).toEqual(2);
      expect(stdout).toBeFalsy();
    } else {
      expect(exitCode).toEqual(0);
      expect(stderr).toBeFalsy();
      // default entry should be used
      expect(stdout).toContain("./src/index.js");
      // should pick up the output path from config
      expect(stdout).toContain("test-output");
      expect(stdout).toContain("compiled successfully");
      // check that the output file exists
      expect(fs.existsSync(path.join(__dirname, "/dist/test-output.js"))).toBeTruthy();
    }
  });
});
