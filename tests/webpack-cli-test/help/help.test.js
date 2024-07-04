"use strict";

const { run, normalizeStderr, normalizeStdout } = require("../utils/test-utils");

describe("help", () => {
  it('should show help information using the "--help" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it.skip('should show help information using the "--help" option with the "verbose" value', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "verbose"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it.skip('should show help information using the "--help" option with the "verbose" value #2', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help=verbose"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should show help information using command syntax", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show the same information using the "--help" option and command syntax', async () => {
    const {
      exitCode: exitCodeFromOption,
      stderr: stderrFromOption,
      stdout: stdoutFromOption,
    } = await run(__dirname, ["--help"]);
    const {
      exitCode: exitCodeFromCommandSyntax,
      stderr: stderrFromCommandSyntax,
      stdout: stdoutFromCommandSyntax,
    } = await run(__dirname, ["help"]);

    expect(exitCodeFromOption).toBe(0);
    expect(exitCodeFromCommandSyntax).toBe(0);
    expect(normalizeStderr(stderrFromOption)).toMatchSnapshot("stderr from option");
    expect(normalizeStderr(stderrFromCommandSyntax)).toMatchSnapshot("stderr from command syntax");
    expect(stdoutFromOption).toBe(stdoutFromCommandSyntax);
    expect(normalizeStdout(stdoutFromOption)).toMatchSnapshot("stdout from option");
    expect(normalizeStdout(stdoutFromCommandSyntax)).toMatchSnapshot("stdout from command syntax");
  });

  it('should show help information and respect the "--color" flag using the "--help" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "--color"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).toContain("\x1b[1m");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information and respect the "--no-color" flag using the "--help" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "--no-color"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  const commands = [
    {
      name: "init",
      alias: ["create", "new", "c", "n"],
    },
    {
      name: "info",
      alias: "i",
    },
    {
      name: "loader",
      alias: "l",
    },
    {
      name: "plugin",
      alias: "p",
    },
    {
      name: "configtest",
      alias: "t",
    },
    {
      name: "watch",
      alias: "w",
    },
    {
      name: "serve",
      alias: ["server", "s"],
    },
    {
      name: "build",
      alias: "b",
    },
  ];

  commands.forEach(({ name, alias }) => {
    // TODO fix it
    const needSkip = name === "serve";

    it(`should show help information for '${name}' command using the "--help" option`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [name, "--help"]);

      expect(exitCode).toBe(0);
      expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");

      if (!needSkip) {
        expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
      }
    });

    it.skip(`should show help information for '${name}' command using the "--help verbose" option`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [name, "--help", "verbose"]);

      expect(exitCode).toBe(0);
      expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");

      if (!needSkip) {
        expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
      }
    });

    it(`should show help information for '${name}' command using command syntax`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, ["help", name]);

      expect(exitCode).toBe(0);
      expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");

      if (!needSkip) {
        expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
      }
    });

    it(`should show help information for '${name}' and respect the "--color" flag using the "--help" option`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [name, "--help", "--color"]);

      expect(exitCode).toBe(0);
      expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
      expect(stdout).toContain("\x1b[1m");

      if (!needSkip) {
        expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
      }
    });

    it(`should show help information for '${name}' and respect the "--no-color" flag using the "--help" option`, async () => {
      const { exitCode, stderr, stdout } = await run(__dirname, [name, "--help", "--no-color"]);

      expect(exitCode).toBe(0);
      expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
      expect(stdout).not.toContain("\x1b[1m");

      if (!needSkip) {
        expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
      }
    });

    const aliases = Array.isArray(alias) ? alias : [alias];

    aliases.forEach((alias) => {
      it(`should show help information for '${alias}' command using the "--help" option`, async () => {
        const { exitCode, stderr, stdout } = await run(__dirname, [alias, "--help"]);

        expect(exitCode).toBe(0);
        expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");

        if (!needSkip) {
          expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
        }
      });

      it.skip(`should show help information for '${alias}' command using the "--help verbose" option`, async () => {
        const { exitCode, stderr, stdout } = await run(__dirname, [alias, "--help", "verbose"]);

        expect(exitCode).toBe(0);
        expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");

        if (!needSkip) {
          expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
        }
      });

      it(`should show help information for '${alias}' command using command syntax`, async () => {
        const { exitCode, stderr, stdout } = await run(__dirname, ["help", alias]);

        expect(exitCode).toBe(0);
        expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");

        if (!needSkip) {
          expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
        }
      });
    });
  });

  it("should show help information with options for sub commands", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--help"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information and taking precedence when "--help" and "--version" option using together', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "--version"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --mode" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--mode"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --target" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--target"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --stats" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--stats"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --cache-type" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--cache-type"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --no-stats" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--no-stats"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help serve --mode" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "serve", "--mode"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --color" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--color"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).toContain("\x1b[1m");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --no-color" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--no-color"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help serve --color" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "serve", "--color"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).toContain("\x1b[1m");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help serve --no-color" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "serve", "--no-color"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help --version" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--version"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should show help information using the "help -v" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "-v"]);

    expect(exitCode).toBe(0);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log error for invalid command using the "--help" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "myCommand"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log error for invalid command using the "--help" option #2', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--flag", "--help"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log error for invalid command using the "--help" option #3', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["serve", "--flag", "--help"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for unknown command using command syntax", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "myCommand"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for unknown command using command syntax #2", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "verbose"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for unknown option using command syntax #2", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--made"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for unknown option using command syntax #3", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "serve", "--made"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for unknown option using command syntax #4", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "bui", "--mode"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for invalid command using command syntax #3", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["help", "--mode", "serve"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error for invalid command using command syntax #4", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "help",
      "serve",
      "--mode",
      "--mode",
    ]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log error for invalid flag with the "--help" option', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "--my-flag"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log error for invalid flag with the "--help" option #2', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help", "init", "info"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log error for invalid flag with the "--help" option #2', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--help="]);

    expect(exitCode).toBe(2);
    expect(stderr).toMatchSnapshot();
    expect(stdout).toBeFalsy();
  });
});
