const { join } = require("path");
const { run } = require("../utils/test-utils");

describe("basic usage", () => {
  it("should work", async () => {
    const { exitCode, stdout, stderr } = await run(__dirname, ["info"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
  });

  it("should work and gets more info in project root", async () => {
    const { exitCode, stderr, stdout } = await run(join(__dirname, "../../"), ["info"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("System:");
    expect(stdout).toContain("Monorepos:");
    expect(stdout).toContain("Packages:");
    expect(stdout).toContain("Node");
    expect(stdout).toContain("npm");
    expect(stdout).toContain("Yarn");
    expect(stdout).toContain("pnpm");
  });

  it("shows an appropriate warning on supplying unknown args", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--unknown"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain("Error: Unknown option '--unknown'");
    expect(stdout).toBeFalsy();
  });
});
