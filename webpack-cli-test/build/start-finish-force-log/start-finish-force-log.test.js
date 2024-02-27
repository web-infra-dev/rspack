"use strict";

const { run, runWatch } = require("../../utils/test-utils");

describe("start finish force log", () => {
  it("start finish force log when env is set", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      env: { WEBPACK_CLI_START_FINISH_FORCE_LOG: true },
    });

    expect(exitCode).toBe(0);
    expect(stderr).toContain("Compiler starting...");
    expect(stderr).toContain("Compiler finished");
    expect(stdout).toContain("compiled successfully");
  });

  it("should show name of the config", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--name", "log config"], {
      env: { WEBPACK_CLI_START_FINISH_FORCE_LOG: true },
    });

    expect(exitCode).toBe(0);
    expect(stderr).toContain("Compiler 'log config' starting...");
    expect(stderr).toContain("Compiler 'log config' finished");
    expect(stdout).toContain("compiled successfully");
  });

  it("should work with watch", async () => {
    const { stderr, stdout } = await runWatch(__dirname, ["watch"], {
      env: { WEBPACK_CLI_START_FINISH_FORCE_LOG: true },
      killString: /Compiler finished/,
    });
    expect(stderr).toContain("Compiler starting...");
    expect(stderr).toContain("Compiler finished");
    expect(stdout).toContain("compiled successfully");
  });

  it("should work with multi compiler", async () => {
    const { exitCode, stderr, stdout } = await run(
      __dirname,
      ["--config", "./webpack.config.array.js"],
      {
        env: { WEBPACK_CLI_START_FINISH_FORCE_LOG: true },
      },
    );
    expect(exitCode).toBe(0);
    expect(exitCode).toBe(0);
    expect(stderr).toContain("Compiler 'Gojou' starting...");
    expect(stderr).toContain("Compiler 'Satoru' starting...");
    expect(stderr).toContain("Compiler 'Gojou' finished");
    expect(stderr).toContain("Compiler 'Satoru' finished");
    expect(stdout).toContain("compiled successfully");
  });
});
