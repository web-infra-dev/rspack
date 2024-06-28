const { run } = require("../../../utils/test-utils");

describe("Zero Config tests", () => {
  it("runs when no config is supplied but entry is present", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, []);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // Should be able to find the entry file
    expect(stdout).toContain("./src/index.js");
    // Should output at the default output dir and filename
    expect(stdout).toContain("main.js");
  });
});
