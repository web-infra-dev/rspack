"use strict";

const path = require("path");
// eslint-disable-next-line node/no-unpublished-require
const rimraf = require("rimraf");
const { run } = require("../../utils/test-utils");

describe("cache", () => {
  it("should work", async () => {
    rimraf.sync(
      path.join(__dirname, "../../../node_modules/.cache/webpack/cache-test-default-development"),
    );

    let { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.config.js"]);

    expect(exitCode).toEqual(0);
    expect(stderr.match(/No pack exists at/g)).toHaveLength(1);
    expect(stderr.match(/Stored pack/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();

    ({ exitCode, stderr, stdout } = await run(__dirname, ["-c", "./webpack.config.js"]));

    expect(exitCode).toEqual(0);
    expect(stderr.match(/restore cache container:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content metadata:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content \d+ \(.+\):/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();
  });

  it("should work in multi compiler mode", async () => {
    rimraf.sync(
      path.join(__dirname, "../../../node_modules/.cache/webpack/cache-test-first-development"),
    );
    rimraf.sync(
      path.join(__dirname, "../../../node_modules/.cache/webpack/cache-test-second-development"),
    );

    let { exitCode, stderr, stdout } = await run(__dirname, ["-c", "./multi.config.js"]);

    expect(exitCode).toEqual(0);
    expect(stderr.match(/No pack exists at/g)).toHaveLength(2);
    expect(stderr.match(/Stored pack/g)).toHaveLength(2);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();

    ({ exitCode, stderr, stdout } = await run(__dirname, ["-c", "./multi.config.js"]));

    expect(exitCode).toEqual(0);
    expect(stderr.match(/restore cache container:/g)).toHaveLength(2);
    expect(stderr.match(/restore cache content metadata:/g)).toHaveLength(2);
    expect(stderr.match(/restore cache content \d+ \(.+\):/g)).toHaveLength(2);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();
  });

  it("should work in multi compiler mode with the `--config-name` argument", async () => {
    rimraf.sync(
      path.join(__dirname, "../../../node_modules/.cache/webpack/cache-test-third-development"),
    );

    let { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./multi.config.js",
      "--config-name",
      "cache-test-first",
      "--name",
      "cache-test-third",
    ]);

    expect(exitCode).toEqual(0);
    expect(stderr.match(/No pack exists at/g)).toHaveLength(1);
    expect(stderr.match(/Stored pack/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();

    ({ exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./multi.config.js",
      "--config-name",
      "cache-test-first",
      "--name",
      "cache-test-third",
    ]));

    expect(exitCode).toEqual(0);
    expect(stderr.match(/restore cache container:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content metadata:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content \d+ \(.+\):/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();
  });

  it("should work with the `--merge` argument", async () => {
    rimraf.sync(
      path.join(__dirname, "../../../node_modules/.cache/webpack/cache-test-fourth-development"),
    );

    let { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./multi.config.js",
      "-c",
      "./webpack.config.js",
      "--merge",
      "--name",
      "cache-test-fourth",
    ]);

    expect(exitCode).toEqual(0);
    expect(stderr.match(/No pack exists at/g)).toHaveLength(1);
    expect(stderr.match(/Stored pack/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();

    ({ exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./multi.config.js",
      "-c",
      "./webpack.config.js",
      "--merge",
      "--name",
      "cache-test-fourth",
    ]));

    expect(exitCode).toEqual(0);
    expect(stderr.match(/restore cache container:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content metadata:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content \d+ \(.+\):/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();
  });

  it("should work with the `--config-name` and `--merge` argument", async () => {
    rimraf.sync(
      path.join(__dirname, "../../../node_modules/.cache/webpack/cache-test-fifth-development"),
    );

    let { exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./multi.config.js",
      "-c",
      "./webpack.config.js",
      "--merge",
      "--config-name",
      "cache-test-first",
      "--config-name",
      "cache-test-second",
      "--name",
      "cache-test-fifth",
    ]);

    expect(exitCode).toEqual(0);
    expect(stderr.match(/No pack exists at/g)).toHaveLength(1);
    expect(stderr.match(/Stored pack/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();

    ({ exitCode, stderr, stdout } = await run(__dirname, [
      "-c",
      "./multi.config.js",
      "-c",
      "./webpack.config.js",
      "--merge",
      "--config-name",
      "cache-test-first",
      "--config-name",
      "cache-test-second",
      "--name",
      "cache-test-fifth",
    ]));

    expect(exitCode).toEqual(0);
    expect(stderr.match(/restore cache container:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content metadata:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content \d+ \(.+\):/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();
  });

  it("should work with autoloading configuration", async () => {
    rimraf.sync(
      path.join(
        __dirname,
        "../../../node_modules/.cache/webpack/cache-test-autoloading-development",
      ),
    );

    let { exitCode, stderr, stdout } = await run(__dirname, ["--name", "cache-test-autoloading"]);

    expect(exitCode).toEqual(0);
    expect(stderr.match(/No pack exists at/g)).toHaveLength(1);
    expect(stderr.match(/Stored pack/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();

    ({ exitCode, stderr, stdout } = await run(__dirname, ["--name", "cache-test-autoloading"]));

    expect(exitCode).toEqual(0);
    expect(stderr.match(/restore cache container:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content metadata:/g)).toHaveLength(1);
    expect(stderr.match(/restore cache content \d+ \(.+\):/g)).toHaveLength(1);
    expect(stderr).toBeTruthy();
    expect(stdout).toBeTruthy();
  });
});
