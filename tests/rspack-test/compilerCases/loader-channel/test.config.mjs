import assert from "node:assert/strict";
import path from "node:path";
import { rspack } from "@rspack/core";
import { runNodeCase } from "@rspack/test-tools/helper/node-case";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

function createCompiler(name, fail = false) {
  const compiler = rspack({
    name,
    context: import.meta.dirname,
    mode: "development",
    cache: false,
    incremental: false,
    devtool: false,
    entry: "./entry.js",
    output: { path: path.join(import.meta.dirname, "output", name) },
    module: {
      rules: [
        {
          test: /entry\.js$/,
          use: [
            {
              loader: path.join(import.meta.dirname, "loader.mjs"),
              options: { name, fail },
            },
          ],
        },
      ],
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap("LoaderChannel", (compilation) => {
            compiler.webpack.NormalModule.getCompilationHooks(
              compilation,
            ).loader.tap("LoaderChannel", (context) => {
              context.channelCompiler = compiler.name;
            });
          });
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  return compiler;
}

async function build(compiler, fail = false) {
  const stats = await runCompiler(compiler);
  const errors = stats.toString({ all: false, errors: true });
  assert.equal(stats.hasErrors(), fail, errors);
  if (fail) assert.match(errors, /intentional channel error: failing/);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    name: "shared-channel",
    description:
      "should dispatch nested loaders concurrently and isolate compiler lifecycles",
    async run() {
      const first = createCompiler("first");
      const second = createCompiler("second");
      const failing = createCompiler("failing", true);
      const open = new Set([first, second, failing]);
      try {
        await Promise.all([build(first), build(second)]);
        // Re-enter immediately while the previous receive is being woken up.
        await Promise.all([build(first), build(second)]);
        await closeCompiler(first);
        open.delete(first);
        await Promise.all([build(failing, true), build(second)]);
        await closeCompiler(failing);
        open.delete(failing);
        await build(second);
      } finally {
        await Promise.all([...open].map(closeCompiler));
      }
    },
  },
  {
    name: "idle-exit",
    description: "should let Node exit while an unclosed compiler is idle",
    async run() {
      await runNodeCase(new URL("./idle.mjs", import.meta.url));
    },
  },
];
