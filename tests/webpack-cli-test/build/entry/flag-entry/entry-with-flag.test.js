"use strict";

const { run, readFile } = require("../../../utils/test-utils");
const { existsSync } = require("fs");
const { resolve } = require("path");

describe("entry flag", () => {
  it("should resolve the path to src/index.cjs", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--entry", "./src/index.cjs"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });

  it("should load ./src/a.js as entry", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--entry", "./src/a.js"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/main.js"))).toBeTruthy();

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).not.toContain("Hello from index.cjs");
  });

  it("should resolve the path to /src/a.js as ./src/a.js", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--entry", "/src/a.js"]);
    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/main.js"))).toBeTruthy();

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).not.toContain("Hello from index.cjs");
  });

  it("should throw error for invalid entry file", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--entry", "./src/test.js"]);

    expect(exitCode).toEqual(1);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("Module not found: Error: Can't resolve");
  });

  it("should reset the `entry` option when the `--entry-reset` is used", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "entry.config.js",
      "--entry-reset",
      "--entry",
      "./src/a.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("{ main: { import: [ './src/a.js' ] } }");
    expect(existsSync(resolve(__dirname, "./dist/main.js"))).toBeTruthy();

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).not.toContain("Hello from index.cjs");
  });

  it("should push ./src/a.js to the entry when --entry-reset is not used", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "entry.config.js",
      "--entry",
      "./src/a.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("{ main: { import: [ './src/index.cjs', './src/a.js' ] } }");

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).toContain("Hello from index.cjs");
  });

  it("should resolve the path to /src/a.js as ./src/a.js", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "entry.config.js",
      "--entry-reset",
      "--entry",
      "/src/a.js",
    ]);
    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
    expect(existsSync(resolve(__dirname, "./dist/main.js"))).toBeTruthy();

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).not.toContain("Hello from index.cjs");
  });

  it("should allow adding multiple entries with --entry-reset", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "entry.config.js",
      "--entry-reset",
      "--entry",
      "./src/a.js",
      "--entry",
      "./src/index.cjs",
    ]);
    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("{ main: { import: [ './src/a.js', './src/index.cjs' ] } }");
    expect(existsSync(resolve(__dirname, "./dist/main.js"))).toBeTruthy();

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/main.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    expect(data).toContain("Hello from a.js");
    expect(data).toContain("Hello from index.cjs");
  });

  it("should throw error if same entry is used via --entry without using --entry-reset", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "entry.config.js",
      "--entry",
      "./src/index.cjs",
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain(
      "configuration.entry should not contain the item './src/index.cjs' twice",
    );
    expect(stdout).toBeFalsy();
  });

  it("should throw error for invalid entry file with --entry-reset", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--config",
      "entry.config.js",
      "--entry-reset",
      "--entry",
      "./src/test.js",
    ]);

    expect(exitCode).toEqual(1);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("Module not found: Error: Can't resolve");
  });
});
