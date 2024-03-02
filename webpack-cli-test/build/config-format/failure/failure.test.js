const path = require("path");

const { run } = require("../../../utils/test-utils");

describe("failure", () => {
  it("should log error on not installed registers", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-c", "webpack.config.iced"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(
      `Failed to load '${path.resolve(__dirname, "./webpack.config.iced")}'`,
    );
    expect(stdout).toBeFalsy();
  });
});
