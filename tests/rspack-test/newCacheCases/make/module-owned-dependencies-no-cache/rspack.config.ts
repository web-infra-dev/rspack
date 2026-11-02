import type { Compilation, Stats } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

// Five runs on one compiler, followed by the harness's automatic restart.
const selectedByBuild = ['a', 'a', 'b', 'b', 'a', 'a'];
const expectedBuiltByBuild = [1, 1, 1, 1, 1, 1];

let buildIndex = 0;
let built = 0;

function assertGraph(compilation: Compilation, selected: string) {
  const entry = [...compilation.modules].find(
    (module) =>
      (module as { resource?: string }).resource?.replace(/^.*[\\/]/, '') ===
      'index.js',
  )!;
  expect(entry).toBeDefined();
  const dependency = entry.dependencies.find(
    (dep) => dep.request === `./${selected}.js`,
  );
  expect(dependency).toBeDefined();
  expect(
    (
      compilation.moduleGraph.getModule(dependency!) as
        { resource?: string } | undefined
    )?.resource?.replace(/^.*[\\/]/, ''),
  ).toBe(`${selected}.js`);
  const blocks = (
    entry as unknown as {
      blocks: { dependencies: { request: string }[] }[];
    }
  ).blocks;
  expect(blocks).toHaveLength(1);
  expect(blocks[0].dependencies).toHaveLength(1);
  const asyncDependency = blocks[0].dependencies[0];
  expect(asyncDependency.request).toBe(`./${selected}-async.js`);
  expect(
    (
      compilation.moduleGraph.getModule(asyncDependency as never) as
        { resource?: string } | undefined
    )?.resource?.replace(/^.*[\\/]/, ''),
  ).toBe(`${selected}-async.js`);
}

export default defineConfig({
  entry: './index.js',
  target: 'node',
  mode: 'production',
  devtool: false,
  incremental: false,
  cache: false,
  experiments: {
    newCache: {
      module: true,
      loader: false,
      codeGeneration: false,
      devtool: false,
      minimize: false,
    },
  },
  output: {
    filename: 'main.js',
    chunkFilename: '[id].[contenthash].js',
    library: { type: 'commonjs2' },
  },
  optimization: {
    concatenateModules: true,
    inlineExports: false,
    minimize: false,
  },
  plugins: [
    definePlugin({
      name: 'ModuleOwnedDependenciesTest',
      apply(compiler) {
        this.compiler = compiler;
        compiler.hooks.compilation.tap(
          'ModuleOwnedDependenciesTest',
          (compilation) => {
            compilation.hooks.finishModules.tap(
              'ModuleOwnedDependenciesTest',
              () => {
                assertGraph(compilation, selectedByBuild[buildIndex]);
              },
            );
            compilation.hooks.buildModule.tap(
              'ModuleOwnedDependenciesTest',
              (module) => {
                if (
                  (module as { resource?: string }).resource?.replace(
                    /^.*[\\/]/,
                    '',
                  ) === 'index.js'
                ) {
                  built++;
                }
              },
            );
          },
        );
        compiler.hooks.done.tap(
          'ModuleOwnedDependenciesTest',
          (stats: Stats) => {
            expect(stats.toJson({ all: false, errors: true }).errors).toEqual(
              [],
            );
            expect(
              String(stats.compilation.getAsset('main.js')!.source.source()),
            ).toContain(
              `CONCATENATED MODULE: ./${selectedByBuild[buildIndex]}.js`,
            );
            expect(built).toBe(expectedBuiltByBuild[buildIndex]);
            built = 0;
            buildIndex++;
          },
        );
      },
    }),
  ],
});
