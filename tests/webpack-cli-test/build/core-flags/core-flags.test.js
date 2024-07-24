"use strict";

const { resolve } = require("path");
const { run, isWindows } = require("../../utils/test-utils");

describe("core flags", () => {
  describe("boolean type flags", () => {
    it("should set bail to true", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--bail"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("bail: true");
    });

    it("should set bail to false", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--no-bail"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("bail: false");
    });
  });

  describe("RegExp type flags", () => {
    it("should ignore the warning emitted", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--ignore-warnings",
        /Generated Warning/,
        "--config",
        "warning.config.js",
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).not.toContain("Module Warning (from ./my-warning-loader.js):");
      expect(stdout).not.toContain("Generated Warning");
    });

    it("should reset options.ignoreWarnings", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--ignore-warnings",
        /Generated Warning/,
        "--ignore-warnings-reset",
        "--config",
        "warning.config.js",
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("Module Warning (from ./my-warning-loader.js):");
      expect(stdout).toContain("Generated Warning");
    });

    it("should throw error for an invalid value", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--ignore-warnings", "abc"]);

      expect(exitCode).toBe(2);
      expect(stderr).toContain(`Invalid value 'abc' for the '--ignore-warnings' option`);
      expect(stderr).toContain(`Expected: 'regular expression (example: /ab?c*/)'`);
      expect(stdout).toBeFalsy();
    });
  });

  describe("reset type flags", () => {
    it("should reset entry correctly", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--entry-reset",
        "--entry",
        "./src/entry.js",
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("src/entry.js");
      expect(stdout).not.toContain("src/main.js");
    });

    it("should throw error if entry is an empty array", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--entry-reset"]);

      expect(exitCode).toBe(2);
      expect(stderr).toContain("Invalid configuration object");
      expect(stdout).toBeFalsy();
    });
  });

  describe("number type flags", () => {
    it("should set parallelism option correctly", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--parallelism", 10]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("parallelism: 10");
    });

    it("should set parallelism option correctly using `=`", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--parallelism=10"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("parallelism: 10");
    });
  });

  describe("enum type flags", () => {
    it("should not allow `true` for amd", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--amd"]);

      expect(exitCode).toBe(2);
      expect(stderr).toContain(`Invalid value 'true' for the '--amd' option`);
      expect(stderr).toContain(`Expected: 'false'`);
      expect(stdout).toBeFalsy();
    });

    it("should allow `false` for amd", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--no-amd"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain("amd: false");
    });

    it("should correctly set `infrastructureLogging.level`", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--infrastructure-logging-level",
        "verbose",
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toContain(`Compiler 'compiler' starting...`);
      expect(stdout).toContain("level: 'verbose'");
    });

    it("should throw error for invalid `infrastructureLogging.level`", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--infrastructure-logging-level",
        "test",
      ]);

      expect(exitCode).toBe(2);
      expect(stderr).toContain(
        `Invalid value 'test' for the '--infrastructure-logging-level' option`,
      );
      expect(stderr).toContain(`Expected: 'none | error | warn | info | log | verbose'`);
      expect(stdout).toBeFalsy();
    });
  });

  describe("path type flags", () => {
    it("should set context option correctly", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--context", "./"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();

      if (isWindows) {
        const windowsPath = resolve(__dirname, "./").replace(/\\/g, "\\\\");
        expect(stdout).toContain(`'${windowsPath}'`);
      } else {
        expect(stdout).toContain(`'${resolve(__dirname, "./")}'`);
      }
    });

    it("should throw module not found error for invalid context", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--context",
        "/invalid-context-path",
      ]);

      expect(exitCode).toBe(1);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`Module not found: Error: Can't resolve './src/main.js'`);
    });
  });

  describe("string type flags", () => {
    it("should set dependencies option correctly", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--dependencies", "lodash"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`dependencies: [ 'lodash' ]`);
    });

    it("should allow to set multiple dependencies", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--dependencies",
        "lodash",
        "react",
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`dependencies: [ 'lodash', 'react' ]`);
    });
  });

  describe("flags with multiple types", () => {
    it("should allow string value for `infrastructureLogging.debug`", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--infrastructure-logging-debug",
        "MyPlugin",
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`debug: [ 'MyPlugin' ]`);
    });

    it("should allow RegExp value for `infrastructureLogging.debug`", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--infrastructure-logging-debug",
        /MyPlugin/,
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`debug: [ /MyPlugin/ ],`);
    });

    it("should allow multiple values for `infrastructureLogging.debug`", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [
        "--infrastructure-logging-debug",
        "MyPlugin",
        /MyAnotherPlugin/,
      ]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`debug: [ 'MyPlugin', /MyAnotherPlugin/ ]`);
    });

    it("should allow string value devtool option", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--devtool", "source-map"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`devtool: 'source-map'`);
    });

    it("should allow string value devtool option using alias", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["-d", "source-map"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`devtool: 'source-map'`);
    });

    it("should allow string value devtool option using alias #1", async () => {
      // cSpell:ignore dsource
      const { exitCode, stderr, stdout } = await run(__dirname, ["-dsource-map"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`devtool: 'source-map'`);
    });

    it("should allow --no-devtool", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--no-devtool"]);

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`devtool: false`);
    });

    it("should log error for invalid devtool value", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--devtool", "invalid"]);

      expect(exitCode).toBe(2);
      expect(stderr).toContain("Invalid configuration object");
      expect(stdout).toBeFalsy();
    });
  });
});
