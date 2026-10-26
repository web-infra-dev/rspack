import fs from "node:fs";
import path from "node:path";

const EFFECT = "inherited-metadata-effect";

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

function expectEffect(compilation, expected) {
  expect(compilation.errors).toHaveLength(0);
  const source = compilation.getAsset("main.js").source.source().toString();
  expect(source.includes(EFFECT)).toBe(expected);
}

function rebuild(compiler, changed, removed = []) {
  return new Promise((resolve, reject) => {
    compiler.__internal__rebuild(new Set(changed), new Set(removed), error => {
      if (error) return reject(error);
      resolve(compiler._lastCompilation);
    });
  });
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "should refresh positive and negative inherited sideEffects lookups on rebuild",
  options(context) {
    const root = context.getDist("src");
    fs.rmSync(root, { recursive: true, force: true });
    write(
      path.join(root, "index.js"),
      'import { value } from "./node_modules/pkg/esm/index.js"; console.log(value);'
    );
    write(
      path.join(root, "node_modules/pkg/package.json"),
      '{"name":"pkg","sideEffects":false}'
    );
    write(path.join(root, "node_modules/pkg/esm/package.json"), '{"type":"module"}');
    write(
      path.join(root, "node_modules/pkg/esm/index.js"),
      'import "./effect.js"; export const value = 42;'
    );
    write(
      path.join(root, "node_modules/pkg/esm/effect.js"),
      `globalThis.effect = ${JSON.stringify(EFFECT)};`
    );
    return {
      context: root,
      entry: "./index.js",
      mode: "development",
      devtool: false,
      cache: false,
      // Isolate the plugin's lookup cache from incremental artifact recovery.
      incremental: false,
      output: { path: context.getDist("output"), filename: "main.js" },
      optimization: { sideEffects: true, usedExports: true, minimize: false }
    };
  },
  async build(context, compiler) {
    const initial = await new Promise((resolve, reject) => {
      compiler.run((error, stats) => {
        if (error) return reject(error);
        resolve(stats.compilation);
      });
    });
    expectEffect(initial, false);

    const metadata = context.getDist("src/node_modules/pkg/package.json");
    write(metadata, '{"name":"pkg","sideEffects":true}');
    expectEffect(await rebuild(compiler, [metadata]), true);

    fs.unlinkSync(metadata);
    const missing = await rebuild(compiler, [], [metadata]);
    expectEffect(missing, true);
    expect(missing.missingDependencies.has(metadata)).toBe(true);

    write(metadata, '{"name":"pkg","sideEffects":false}');
    const restored = await rebuild(compiler, [metadata]);
    expectEffect(restored, false);
    expect(restored.fileDependencies.has(metadata)).toBe(true);
  }
};
