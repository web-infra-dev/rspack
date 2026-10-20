import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";

// Every invocation runs in a fresh Node process; close() must flush the producer.
export default async function build(root, settings) {
  const built = [];
  const reused = [];
  const cache = {
    type: "filesystem",
    cacheLocation: path.join(root, settings.cache),
    version: settings.version ?? settings.cache,
    moduleProfile: settings.moduleProfile,
    loaderProfile: settings.loaderProfile,
    ...(settings.source && {
      readonlyFrom: {
        cacheLocation: path.join(root, settings.source),
        version: settings.sourceVersion ?? "app-cache",
      },
    }),
    buildDependencies: settings.producer
      ? { config: [path.join(root, "build-dependency.json")] }
      : {},
  };
  const output = path.join(root, `output-${settings.cache}`);
  const compiler = rspack({
    context: root,
    entry: `./${settings.entry ?? "test.js"}`,
    mode: "development",
    target: "node",
    devtool: false,
    incremental: false,
    node: { __filename: false, __dirname: false },
    output: {
      path: output,
      filename: "main.cjs",
      library: { type: "commonjs2" },
    },
    optimization: { concatenateModules: false, minimize: false },
    cache,
    snapshot: { module: { hash: true, timestamp: false } },
    experiments: { newCache: { module: settings.moduleCache ?? true } },
    module: {
      rules: [
        {
          test: /\.js$/,
          parser: settings.parser,
          use: [
            {
              loader: path.join(root, "loader.cjs"),
              options: { tag: settings.tag ?? "one" },
              cache: true,
            },
          ],
        },
      ],
    },
    plugins: [
      ...(settings.rstest
        ? [
            new rspack.experiments.RstestPlugin({
              injectModulePathName: true,
              importMetaPathName: true,
              hoistMockModule: false,
              manualMockRoot: path.join(root, "__mocks__"),
            }),
          ]
        : []),
      {
        apply(compiler) {
          compiler.hooks.compilation.tap("ReuseProbe", (compilation) => {
            compilation.hooks.buildModule.tap("ReuseProbe", (module) => {
              if (module.resource) built.push(path.basename(module.resource));
            });
            compilation.hooks.stillValidModule.tap("ReuseProbe", (module) => {
              if (module.resource) reused.push(path.basename(module.resource));
            });
          });
        },
      },
    ],
  });
  try {
    const stats = await new Promise((resolve, reject) => {
      compiler.run((error, stats) => (error ? reject(error) : resolve(stats)));
    });
    assert.equal(
      stats.hasErrors(),
      false,
      stats.toString({ all: false, errors: true }),
    );
    assert.equal(
      stats.hasWarnings(),
      false,
      stats.toString({ all: false, warnings: true }),
    );
  } finally {
    await new Promise((resolve, reject) =>
      compiler.close((error) => (error ? reject(error) : resolve())),
    );
  }
  const require = createRequire(import.meta.url);
  const result = require(path.join(output, "main.cjs"));
  return {
    built: built.sort(),
    reused: reused.sort(),
    value: result.value ?? null,
    file: result.file ?? null,
    own: result.own ?? null,
    lazy: result.lazy ? await result.lazy() : null,
    loads: fs.existsSync(path.join(root, "loads.jsonl"))
      ? fs
          .readFileSync(path.join(root, "loads.jsonl"), "utf8")
          .trim()
          .split("\n")
      : [],
  };
}
