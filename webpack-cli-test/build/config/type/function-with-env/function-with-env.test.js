"use strict";
const { existsSync } = require("fs");
const { resolve } = require("path");
const { run, readFile, isWindows } = require("../../../../utils/test-utils");

describe("function configuration", () => {
  it("should throw when env is not supplied", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env"]);

    expect(exitCode).toBe(2);
    expect(stderr).toContain("Error: Option '--env <value...>' argument missing");
    expect(stdout).toBeFalsy();
  });

  it("is able to understand a configuration file as a function", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env", "isProd"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("isProd: true");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/prod.js"))).toBeTruthy();
  });

  it("is able to understand a configuration file as a function", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env", "isDev"]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("isDev: true");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/dev.js"))).toBeTruthy();
  });

  it("Supports passing string in env", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "environment=production",
      "--env",
      "app.title=Luffy",
      "-c",
      "webpack.env.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("environment: 'production'");
    expect(stdout).toContain("app: { title: 'Luffy' }");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/Luffy.js"))).toBeTruthy();
  });

  it("Supports long nested values in env", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "file.name.is.this=Atsumu",
      "--env",
      "environment=production",
      "-c",
      "webpack.env.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("environment: 'production'");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/Atsumu.js"))).toBeTruthy();
  });

  it("Supports multiple equal in a string", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "file=name=is=Eren",
      "--env",
      "environment=multipleq",
      "-c",
      "webpack.env.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("environment: 'multipleq'");
    expect(stdout).toContain("file: 'name=is=Eren'");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/name=is=Eren.js"))).toBeTruthy();
  });

  it("Supports dot at the end", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "name.=Hisoka",
      "--env",
      "environment=dot",
      "-c",
      "webpack.env.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("environment: 'dot'");
    expect(stdout).toContain("'name.': 'Hisoka'");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/Hisoka.js"))).toBeTruthy();
  });

  it("Supports dot at the end", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "name.",
      "--env",
      "environment=dot",
      "-c",
      "webpack.env.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("environment: 'dot'");
    expect(stdout).toContain("'name.': true");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/true.js"))).toBeTruthy();
  });

  it("Supports empty string", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env", `foo=''`]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain(`foo: "''"`);
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/empty-string.js"))).toBeTruthy();
  });

  it('Supports empty string with multiple "="', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env", `foo=bar=''`]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain(`foo: "bar=''"`);
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/new-empty-string.js"))).toBeTruthy();
  });

  it('Supports env variable with "=" at the end', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env", `foo=`]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // should log foo: undefined
    expect(stdout).toContain("foo: undefined");
  });

  it('Supports env variable with "foo=undefined" at the end', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--env", `foo=undefined`]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    // should log foo: 'undefined'
    expect(stdout).toContain("foo: 'undefined'");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/undefined-foo.js"))).toBeTruthy();
  });

  // macOS/Linux specific syntax
  if (!isWindows) {
    it("Supports empty string in shell environment", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--env", "foo=\\'\\'"], {
        shell: true,
      });

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`foo: "''"`);
      // Should generate the appropriate files
      expect(existsSync(resolve(__dirname, "./dist/empty-string.js"))).toBeTruthy();
    });
    it("should set the variable to undefined if empty string is not escaped in shell environment", async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["--env", "foo=''"], {
        shell: true,
      });

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      expect(stdout).toContain(`foo: undefined`);
    });
    it('Supports env variable with "=$NON_EXISTENT_VAR" at the end', async () => {
      const { exitCode, stderr, stdout } = await run(
        __dirname,
        ["--env", `foo=$NON_EXISTENT_VAR`],
        {
          shell: true,
        },
      );

      expect(exitCode).toBe(0);
      expect(stderr).toBeFalsy();
      // should log foo: undefined
      expect(stdout).toContain("foo: undefined");
    });
  }

  it("is able to understand multiple env flags", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "isDev",
      "--env",
      "verboseStats",
      "--env",
      "envMessage",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("verboseStats: true");
    expect(stdout).toContain("envMessage: true");
    // check that the verbose env is respected
    expect(stdout).toContain("LOG from webpack");

    let data;

    try {
      data = await readFile(resolve(__dirname, "./dist/dev.js"), "utf-8");
    } catch (error) {
      expect(error).toBe(null);
    }

    // check if the values from DefinePlugin make it to the compiled code
    expect(data).toContain("env message present");
  });

  it("is able to apply last flag with same name", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--env",
      "name.=foo",
      "--env",
      "name.=baz",
      "--env",
      "environment=dot",
      "-c",
      "webpack.env.config.js",
    ]);

    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toContain("'name.': 'baz'");
    // Should generate the appropriate files
    expect(existsSync(resolve(__dirname, "./dist/baz.js"))).toBeTruthy();
  });
});
