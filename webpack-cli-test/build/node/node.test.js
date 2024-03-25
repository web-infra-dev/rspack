"use strict";
const { resolve } = require("path");
const { run } = require("../../utils/test-utils");

describe("node flags", () => {
  it("is able to pass the options flags to node js", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--output-path", "./bin"], {
      nodeOptions: [
        `--require=${resolve(__dirname, "bootstrap.js")}`,
        `--require=${resolve(__dirname, "bootstrap2.js")}`,
      ],
    });

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("---from bootstrap.js---");
    expect(stdout).toContain("---from bootstrap2.js---");
  });

  it("throws an error on supplying unknown flags", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      nodeOptions: ["--unknown"],
    });

    expect(exitCode).not.toBe(0);
    expect(stderr).toContain("bad option");
    expect(stdout).toBeFalsy();
  });

  it("throws an error if no values were supplied with --max-old-space-size", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      nodeOptions: ["--max-old-space-size"],
    });

    expect(exitCode).not.toBe(0);
    expect(stderr).toContain("value for flag --max-old-space-size");
    expect(stdout).toBeFalsy();
  });

  it("throws an error if an illegal value was supplied with --max-old-space-size", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      nodeOptions: ["--max_old_space_size=1024a"],
    });

    expect(exitCode).not.toBe(0);
    expect(stderr).toContain(
      "Error: illegal value for flag --max_old_space_size=1024a of type size_t",
    );
    expect(stdout).toBeFalsy();
  });
});
