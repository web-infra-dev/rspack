"use strict";

const { existsSync } = require("fs");
const { join, resolve } = require("path");
const {
  run,
  runPromptWithAnswers,
  uniqueDirectoryForTest,
  normalizeStdout,
  normalizeStderr,
} = require("../utils/test-utils");

const firstPrompt = "? Loader name (my-loader)";
const ENTER = "\x0D";
const DOWN = "\x1B\x5B\x42";

const dataForTests = (rootAssetsPath) => ({
  loaderName: "test-loader",
  loaderPath: join(rootAssetsPath, "test-loader"),
  defaultLoaderPath: join(rootAssetsPath, "my-loader"),
  genPath: join(rootAssetsPath, "test-assets"),
  customLoaderPath: join(rootAssetsPath, "test-assets", "loaderName"),
  defaultTemplateFiles: [
    "package.json",
    "package-lock.json",
    "examples",
    "src",
    "test",
    "src/index.js",
    "examples/simple/webpack.config.js",
  ],
});

describe("loader command", () => {
  it.only("should ask the loader name when invoked", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { stdout, stderr } = await runPromptWithAnswers(assetsPath, ["loader"]);

    expect(stdout).toBeTruthy();
    expect(normalizeStderr(stderr)).toBeFalsy();
    expect(normalizeStdout(stdout)).toContain(firstPrompt);
  });

  it("should scaffold loader with default name if no loader name provided", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { defaultLoaderPath, defaultTemplateFiles } = dataForTests(assetsPath);
    let { stdout } = await runPromptWithAnswers(assetsPath, ["loader"], [ENTER, ENTER]);

    expect(normalizeStdout(stdout)).toContain(firstPrompt);

    // Skip test in case installation fails
    if (!existsSync(resolve(defaultLoaderPath, "./package-lock.json"))) {
      return;
    }

    // Check if the output directory exists with the appropriate loader name
    expect(existsSync(defaultLoaderPath)).toBeTruthy();

    // All test files are scaffolded
    defaultTemplateFiles.forEach((file) => {
      expect(existsSync(defaultLoaderPath, file)).toBeTruthy();
    });

    // Check if the generated loader works successfully
    const path = resolve(defaultLoaderPath, "./examples/simple/");

    ({ stdout } = await run(path, []));

    expect(stdout).toContain("my-loader");
  });

  it("should scaffold loader template with a given name", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { loaderName, loaderPath, defaultTemplateFiles } = dataForTests(assetsPath);
    let { stdout } = await runPromptWithAnswers(
      assetsPath,
      ["loader"],
      [`${loaderName}${ENTER}`, ENTER],
    );

    expect(normalizeStdout(stdout)).toContain(firstPrompt);

    // Skip test in case installation fails
    if (!existsSync(resolve(loaderPath, "./package-lock.json"))) {
      return;
    }

    // Check if the output directory exists with the appropriate loader name
    expect(existsSync(loaderPath)).toBeTruthy();

    // All test files are scaffolded
    defaultTemplateFiles.forEach((file) => {
      expect(existsSync(loaderPath, file)).toBeTruthy();
    });

    // Check if the generated loader works successfully
    const path = resolve(loaderPath, "./examples/simple/");

    ({ stdout } = await run(path, []));

    expect(stdout).toContain("test-loader");
  });

  it("should scaffold loader template in the specified path", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { loaderName, customLoaderPath, defaultTemplateFiles } = dataForTests(assetsPath);
    let { stdout } = await runPromptWithAnswers(
      assetsPath,
      ["loader", "test-assets"],
      [`${loaderName}${ENTER}`, ENTER],
    );

    expect(normalizeStdout(stdout)).toContain(firstPrompt);

    // Skip test in case installation fails
    if (!existsSync(resolve(customLoaderPath, "./package-lock.json"))) {
      return;
    }

    // Check if the output directory exists with the appropriate loader name
    expect(existsSync(customLoaderPath)).toBeTruthy();

    // All test files are scaffolded
    defaultTemplateFiles.forEach((file) => {
      expect(existsSync(customLoaderPath, file)).toBeTruthy();
    });

    // Check if the generated loader works successfully
    const path = resolve(customLoaderPath, "./examples/simple/");

    ({ stdout } = await run(path, []));

    expect(stdout).toContain("test-loader");
  });

  it("should scaffold loader template in the current directory", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { loaderName, customLoaderPath, defaultTemplateFiles } = dataForTests(assetsPath);

    let { stdout } = await runPromptWithAnswers(
      assetsPath,
      ["loader", "./"],
      [`${loaderName}${ENTER}`, ENTER],
    );

    expect(normalizeStdout(stdout)).toContain(firstPrompt);

    // Skip test in case installation fails
    if (!existsSync(resolve(customLoaderPath, "./package-lock.json"))) {
      return;
    }

    // Check if the output directory exists with the appropriate loader name
    expect(existsSync(customLoaderPath)).toBeTruthy();

    // All test files are scaffolded
    defaultTemplateFiles.forEach((file) => {
      expect(existsSync(customLoaderPath, file)).toBeTruthy();
    });

    // Check if the generated loader works successfully
    const path = resolve(customLoaderPath, "./examples/simple/");

    ({ stdout } = await run(path, []));

    expect(stdout).toContain("test-loader");
  });

  it("should prompt on supplying an invalid template", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { stderr } = await runPromptWithAnswers(assetsPath, ["loader", "--template=unknown"]);

    expect(stderr).toContain("unknown is not a valid template");
  });

  it("recognizes '-t' as an alias for '--template'", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { defaultLoaderPath, defaultTemplateFiles } = dataForTests(assetsPath);
    let { stdout } = await runPromptWithAnswers(
      assetsPath,
      ["loader", "-t", "default"],
      [`${ENTER}`, ENTER],
    );

    expect(normalizeStdout(stdout)).toContain(firstPrompt);

    // Skip test in case installation fails
    if (!existsSync(resolve(defaultLoaderPath, "./package-lock.json"))) {
      return;
    }

    // Check if the output directory exists with the appropriate loader name
    expect(existsSync(defaultLoaderPath)).toBeTruthy();

    // All test files are scaffolded
    defaultTemplateFiles.forEach((file) => {
      expect(existsSync(defaultLoaderPath, file)).toBeTruthy();
    });

    // Check if the generated loader works successfully
    const path = resolve(assetsPath, "./my-loader/examples/simple/");

    ({ stdout } = await run(path, []));

    expect(stdout).toContain("my-loader");
  });

  it("uses yarn as the package manager when opted", async () => {
    const assetsPath = await uniqueDirectoryForTest();
    const { defaultLoaderPath, defaultTemplateFiles } = dataForTests(assetsPath);
    let { stdout } = await runPromptWithAnswers(
      assetsPath,
      ["loader", "-t", "default"],
      [`${ENTER}`, `${DOWN}${ENTER}`],
    );

    expect(normalizeStdout(stdout)).toContain(firstPrompt);

    // Skip test in case installation fails
    if (!existsSync(resolve(defaultLoaderPath, "./yarn.lock"))) {
      return;
    }

    // Check if the output directory exists with the appropriate loader name
    expect(existsSync(defaultLoaderPath)).toBeTruthy();

    // All test files are scaffolded
    const files = [
      ...defaultTemplateFiles.filter((file) => file !== "package-lock.json"),
      "yarn.lock",
    ];

    files.forEach((file) => {
      expect(existsSync(defaultLoaderPath, file)).toBeTruthy();
    });

    // Check if the generated loader works successfully
    const path = resolve(assetsPath, "./my-loader/examples/simple/");

    ({ stdout } = await run(path, []));

    expect(stdout).toContain("my-loader");
  });
});
