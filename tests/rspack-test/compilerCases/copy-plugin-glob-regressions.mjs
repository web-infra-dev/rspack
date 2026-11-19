import fs from "node:fs";
import path from "node:path";
import { asset, createProject, write } from "./_copy-plugin-cache.mjs";

function copyCase(name, patterns, setup, check) {
  return {
    description: `should ${name}`,
    options(context) {
      const root = context.getDist(name);
      const project = createProject(context, name, patterns(root), false);
      setup(root);
      return project.options;
    },
    async build(_context, compiler) {
      const stats = await new Promise((resolve, reject) => {
        compiler.run((error, stats) => (error ? reject(error) : resolve(stats)));
      });
      check(stats.compilation, compiler.context);
    },
  };
}

function copiedAssets(compilation) {
  return compilation.getAssets()
    .map(({ name }) => name)
    .filter((name) => name.startsWith("copied/"))
    .sort();
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  copyCase(
    "treat a leading bang in from as a path character",
    () => [{ from: "!assets/*.txt", to: "copied", toType: "dir" }],
    (root) => {
      write(root, "!assets/picked.txt", "picked");
      write(root, "assets/other.txt", "other");
      write(root, ".hidden", "hidden");
    },
    (compilation) => {
      expect(compilation.errors).toHaveLength(0);
      expect(copiedAssets(compilation)).toEqual(["copied/!assets/picked.txt"]);
    },
  ),
  ...[true, false].map((noErrorOnMissing) => copyCase(
    `keep copying valid patterns alongside ${noErrorOnMissing ? "an optional" : "a required"} invalid glob`,
    () => [
      { from: "assets/[", noErrorOnMissing },
      { from: "assets/source.txt", to: "copied/source.txt" },
    ],
    (root) => write(root, "assets/source.txt", "copied"),
    (compilation) => {
      expect(compilation.errors).toHaveLength(noErrorOnMissing ? 0 : 1);
      expect(asset(compilation, "copied/source.txt")).toBe("copied");
    },
  )),
  copyCase(
    "register the literal directory of a case-insensitive absolute glob",
    (root) => [{
      from: path.join(root, "assets", "*.TXT"),
      to: "copied",
      toType: "dir",
      globOptions: { caseSensitiveMatch: false },
    }],
    (root) => write(root, "assets/source.txt", "copied"),
    (compilation, root) => {
      expect(compilation.errors).toHaveLength(0);
      expect(copiedAssets(compilation)).toEqual(["copied/assets/source.txt"]);
      expect([...compilation.contextDependencies]).toEqual([path.join(root, "assets")]);
    },
  ),
  copyCase(
    "match only real directory entries inside braces",
    () => [{ from: "{.,assets}/*.txt", to: "copied", toType: "dir" }],
    (root) => {
      write(root, "root.txt", "root");
      write(root, "assets/nested.txt", "nested");
    },
    (compilation) => {
      expect(compilation.errors).toHaveLength(0);
      expect(copiedAssets(compilation)).toEqual(["copied/assets/nested.txt"]);
    },
  ),
  ...(process.platform === "linux" ? [
    copyCase(
      "follow a symbolic link before resolving its parent segment",
      () => [{ from: "link/../*.txt", to: "copied", toType: "dir" }],
      (root) => {
        write(root, "shared/sub/unused.txt", "unused");
        write(root, "shared/value.txt", "target directory");
        write(root, "value.txt", "project directory");
        fs.symlinkSync("shared/sub", path.join(root, "link"));
      },
      (compilation) => {
        expect(compilation.errors).toHaveLength(0);
        expect(asset(compilation, "copied/value.txt")).toBe("target directory");
      },
    ),
    copyCase(
      "honor caseSensitiveMatch for an existing literal file",
      () => [{
        from: "assets/same.txt",
        to: "copied",
        toType: "dir",
        globOptions: { caseSensitiveMatch: false },
      }],
      (root) => {
        write(root, "assets/same.txt", "one");
        write(root, "assets/saMe.txt", "two");
      },
      (compilation, root) => {
        expect(compilation.errors).toHaveLength(0);
        expect(copiedAssets(compilation)).toEqual(["copied/saMe.txt", "copied/same.txt"]);
        expect([...compilation.fileDependencies]).toContain(path.join(root, "assets", "saMe.txt"));
      },
    ),
    copyCase(
      "treat a colon in a POSIX directory name literally",
      () => [{
        from: "ASSETS:/*.TXT",
        to: "copied",
        toType: "dir",
        globOptions: { caseSensitiveMatch: false },
      }],
      (root) => write(root, "ASSETS:/file.txt", "copied"),
      (compilation) => {
        expect(compilation.errors).toHaveLength(0);
        expect(copiedAssets(compilation)).toEqual(["copied/ASSETS:/file.txt"]);
      },
    ),
    copyCase(
      "report a matched dangling symbolic link alongside valid assets",
      () => [{ from: "assets/*.txt", to: "copied", toType: "dir" }],
      (root) => {
        write(root, "assets/good.txt", "copied");
        fs.symlinkSync("absent.txt", path.join(root, "assets/orphan.txt"));
      },
      (compilation) => {
        expect(compilation.errors).toHaveLength(1);
        expect(compilation.errors[0].message).toContain("os error 2");
        expect(asset(compilation, "copied/assets/good.txt")).toBe("copied");
      },
    ),
  ] : []),
];
