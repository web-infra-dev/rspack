"use strict";

const { run } = require("../../../utils/test-utils");

describe("merge flag configuration", () => {
  it("merges two configurations together", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "./1.js",
      "--config",
      "./2.js",
      "--merge",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("WebpackCLITestPlugin"); // from 1.js
    expect(stdout).toContain("second-output.js"); // from 2.js
  });

  it("merges more than two configurations together", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["--config", "./1.js", "--config", "./2.js", "--config", "./3.js", "--merge"],
      false,
    );

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("WebpackCLITestPlugin"); // from 1.js
    expect(stdout).toContain("target: 'node'"); // from 2.js
    expect(stdout).toContain("third-output.js"); // from 3.js
  });

  it("merges two configurations together with flag alias", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "./1.js",
      "--config",
      "./2.js",
      "-m",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("WebpackCLITestPlugin"); // from 1.js
    expect(stdout).toContain("second-output.js"); // from 2.js
  });

  it("fails when there are less than 2 configurations to merge", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--config", "./1.js", "--merge"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain("At least two configurations are required for merge.");
    expect(stdout).toBeFalsy();
  });
});
