import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
  closeCompiler,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

async function build(context, log, { entry, symlinks, cache = true }) {
  fs.writeFileSync(log, "");
  const compiler = rspack({
    context,
    mode: "development",
    devtool: false,
    entry,
    output: {
      path: path.join(context, cache ? "dist" : "dist-cold"),
      library: { type: "commonjs2" },
    },
    target: "node",
    node: { __dirname: true },
    resolve: { symlinks },
    module: {
      rules: [
        {
          test: /[\\/]src[\\/].*\.js$/,
          use: [
            {
              loader: path.join(context, "count-loader.cjs"),
              options: { log },
            },
          ],
        },
      ],
    },
    cache: cache ? { type: "persistent", portable: true } : false,
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
      root: cache ? compiler.options.cache.portable.root : undefined,
      built: fs.readFileSync(log, "utf-8").split("\n").filter(Boolean),
      cacheLog: (logging?.["rspack.persistentCache"]?.entries ?? []).map(
        entry => entry.message,
      ),
      output: fs.readFileSync(
        path.join(compiler.options.output.path, "main.js"),
        "utf-8",
      ),
    };
  } finally {
    await closeCompiler(compiler);
  }
}

function setup() {
  const base = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-symlink-")),
  );
  const real = path.join(base, "real", "project");
  fs.cpSync(import.meta.dirname, path.join(base, "real"), {
    recursive: true,
    filter: source => !source.endsWith("test.config.mjs"),
  });
  fs.mkdirSync(path.join(real, ".git"));
  fs.symlinkSync(path.join(base, "real"), path.join(base, "link"), "dir");
  return {
    base,
    real,
    linked: path.join(base, "link", "project"),
    log: path.join(base, "builds.log"),
  };
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
  description:
    "should reuse portable cache entries when context is reached through a symlink",
  // Directory symlinks need extra privileges on Windows.
  skip: process.platform === "win32",
  async run() {
    {
      const { base, real, linked, log } = setup();
      try {
        const options = { entry: "./src/index.js", symlinks: true };
        const first = await build(linked, log, options);
        expect(path.relative(base, first.root)).toBe(
          path.join("link", "project"),
        );
        expect(first.built.sort()).toEqual([
          path.join(real, "src", "a.js"),
          path.join(real, "src", "index.js"),
        ]);

        const next = await build(linked, log, options);
        expect(next.built).toEqual([]);
        expect(next.cacheLog).toContain(
          "make persistent cache recovery succeeded",
        );
        expect(
          next.cacheLog.filter(message => /dropped [1-9]/.test(message)),
        ).toEqual([]);
      } finally {
        fs.rmSync(base, { recursive: true, force: true });
      }
    }

    for (const symlinks of [true, false]) {
      const { base, real, linked, log } = setup();
      try {
        const options = { entry: "./src/main.js", symlinks };
        await build(linked, log, options);
        const main = path.join(real, "src", "main.js");
        fs.writeFileSync(
          main,
          fs.readFileSync(main, "utf-8").replace("original", "edited"),
        );

        const next = await build(real, log, options);
        const cold = await build(real, log, { ...options, cache: false });
        expect(next.output).toBe(cold.output);
      } finally {
        fs.rmSync(base, { recursive: true, force: true });
      }
    }
  },
};
