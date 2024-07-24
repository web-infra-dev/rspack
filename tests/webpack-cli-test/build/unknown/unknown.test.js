"use strict";

const { run, normalizeStdout, normalizeStderr } = require("../../utils/test-utils");

describe("unknown behaviour", () => {
  it("should log an error if an unknown flag is passed", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--unknown"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed #2", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-u"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed and includes =", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--unknown=foo"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed #3", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-u", "--unknown"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed #4", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-u", "-u"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed #5", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["-u", "foo"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed using "bundle" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["bundle", "--unknown"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed using "b" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["b", "--unknown"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed using "bundle" command #2', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--unknown", "bundle"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed using "info" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--unknown"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed using "i" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["i", "--unknown"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed using "i" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--unknown", "i"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error and respect --color flag", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--unknown", "--color"]);

    expect(exitCode).toBe(2);
    expect(stderr).toMatchSnapshot("stderr");
    expect(stdout).toMatchSnapshot("stdout");
  });

  it("should log error for unknown flag and respect --no-color", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["--unknown", "--no-color"]);

    expect(exitCode).toBe(2);
    expect(stderr).toMatchSnapshot("stderr");
    expect(stdout).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed and suggests the closest match to an unknown flag", async () => {
    // cSpell:ignore entyr
    const { exitCode, stderr, stdout } = await run(__dirname, ["--entyr", "./a.js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed and suggests the closest match to an unknown flag #2", async () => {
    // cSpell:ignore fileneme
    const { exitCode, stderr, stdout } = await run(__dirname, ["--output-fileneme", "[name].js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log an error if an unknown flag is passed and suggests the closest match to an unknown flag #3", async () => {
    // cSpell:ignore commnjs
    const { exitCode, stderr, stdout } = await run(__dirname, [
      "--output-library-auxiliary-comment-commnjs",
    ]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed and suggests the closest match to an unknown flag using "bundle" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["bundle", "--entyr", "./a.js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed and suggests the closest match to an unknown flag using "b" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["b", "--entyr", "./a.js"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed and suggests the closest match to an unknown flag using "info" command', async () => {
    // cSpell:ignore outpyt
    const { exitCode, stderr, stdout } = await run(__dirname, ["info", "--outpyt"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it('should log an error if an unknown flag is passed and suggests the closest match to an unknown flag using "i" command', async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["i", "--outpyt"]);

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error if an unknown command passed", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, ["qqq"], true, [], {
      TERM_PROGRAM: false,
    });

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });

  it("should log error and provide suggestion if an unknown command passed", async () => {
    // cSpell:ignore serverr
    const { exitCode, stderr, stdout } = await run(__dirname, ["serverr"], true, [], {
      TERM_PROGRAM: false,
    });

    expect(exitCode).toBe(2);
    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(normalizeStdout(stdout)).toMatchSnapshot("stdout");
  });
});
