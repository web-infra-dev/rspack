"use strict";

const { run } = require("../../utils/test-utils");

describe("extends property", () => {
  it("extends a provided webpack config correctly", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/simple-case");

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("base.webpack.config.js");
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("name: 'base_config'");
    expect(stdout).toContain("mode: 'development'");
  });

  it("extends a provided array of webpack configs correctly", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/multiple-extends");

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("base1.webpack.config.js");
    expect(stdout).toContain("base2.webpack.config.js");
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("name: 'base_config2'");
    expect(stdout).toContain("mode: 'development'");
  });

  it("extends a multilevel config correctly", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/multi-level-extends");

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("base1.webpack.config.js");
    expect(stdout).toContain("base2.webpack.config.js");
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("name: 'base_config1'");
    expect(stdout).toContain("mode: 'production'");
  });

  it("extends a provided webpack config for multiple configs correctly", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/multiple-configs");

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("base.webpack.config.js");
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("name: 'derived_config1'");
    expect(stdout).toContain("name: 'derived_config2'");
    expect(stdout).not.toContain("name: 'base_config'");
    expect(stdout).toContain("mode: 'development'");
    expect(stdout).toContain("topLevelAwait: true");
  });

  it("extends a provided webpack config for multiple configs correctly #2", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/multiple-configs2");

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("base.webpack.config.js");
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("name: 'base_config'");
    expect(stdout).toContain("name: 'derived_config2'");
    expect(stdout).toContain("mode: 'development'");
    expect(stdout).toContain("topLevelAwait: true");
  });

  it("multiple extends a provided webpack config passed in the cli correctly", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/extends-cli-option", [
      "--extends",
      "./base.webpack.config.js",
      "--extends",
      "./deep.base.webpack.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("base.webpack.config.js");
    expect(stdout).toContain("deep.base.webpack.config.js");
    expect(stdout).toContain("name: 'base_config'");
    expect(stdout).toContain("mode: 'development'");
    expect(stdout).toContain("bail: true");
  });

  it("should work with multiple extends and multiple configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/multiple-configs1", [
      "--extends",
      "./base.webpack.config.js",
      "--extends",
      "./other.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("base.webpack.config.js");
    expect(stdout).toContain("other.config.js");
    expect(stdout).toContain("name: 'derived_config1'");
    expect(stdout).toContain("name: 'derived_config2'");
    expect(stdout).toContain("topLevelAwait: true");
  });

  it("CLI `extends` should override `extends` in a configuration", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/simple-case", [
      "--extends",
      "./override.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("override.config.js");
    expect(stdout).toContain("derived.webpack.config.js");
    expect(stdout).toContain("name: 'override_config'");
    expect(stdout).toContain("mode: 'development'");
  });

  it("should throw an error on recursive", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/recursive-extends");

    expect(exitCode).toBe(2);
    expect(stderr).toContain("Recursive configuration detected, exiting");
    expect(stdout).toBeFalsy();
  });

  it("should throw an error on recursive #2", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname + "/recursive-extends", [
      "--config",
      "other.config.js",
    ]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain("Recursive configuration detected, exiting");
    expect(stdout).toBeFalsy();
  });
});
