import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
  closeCompiler,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

async function build(context, log) {
  fs.writeFileSync(log, "");
  const compiler = rspack({
    context,
    mode: "development",
    devtool: false,
    entry: "./src/index.js",
    output: { path: path.join(context, "dist") },
    module: {
      rules: [
        {
          test: /\.js$/,
          use: [
            {
              loader: path.join(context, "count-loader.cjs"),
              options: { log },
            },
          ],
        },
      ],
    },
    cache: { type: "persistent", portable: true },
  });
  try {
    const stats = await runCompiler(compiler);
    expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
    const { logging } = stats.toJson({
      all: false,
      logging: false,
      loggingDebug: /^rspack\.persistentCache$/,
    });
    return {
      root: compiler.options.cache.portable.root,
      built: fs.readFileSync(log, "utf-8").split("\n").filter(Boolean),
      cacheLog: (logging["rspack.persistentCache"]?.entries ?? []).map(
        entry => entry.message,
      ),
    };
  } finally {
    await closeCompiler(compiler);
  }
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description:
    "should reuse portable cache entries when context is reached through a symlink",
  // Directory symlinks need extra privileges on Windows.
  skip: process.platform === "win32",
  async run() {
    const base = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-symlink-")),
    );
    try {
      const real = path.join(base, "real", "project");
      fs.cpSync(path.join(import.meta.dirname, "project"), real, {
        recursive: true,
      });
      fs.mkdirSync(path.join(real, ".git"));
      fs.symlinkSync(path.join(base, "real"), path.join(base, "link"), "dir");
      const linked = path.join(base, "link", "project");
      const log = path.join(base, "builds.log");

      const first = await build(linked, log);
      expect(path.relative(base, first.root)).toBe(
        path.join("link", "project"),
      );
      expect(first.built.sort()).toEqual([
        path.join(real, "src", "a.js"),
        path.join(real, "src", "index.js"),
      ]);

      const next = await build(linked, log);
      expect(next.built).toEqual([]);
      expect(next.cacheLog).toContain("make persistent cache recovery succeeded");
      expect(
        next.cacheLog.filter(message => /dropped [1-9]/.test(message)),
      ).toEqual([]);
    } finally {
      fs.rmSync(base, { recursive: true, force: true });
    }
  },
};
