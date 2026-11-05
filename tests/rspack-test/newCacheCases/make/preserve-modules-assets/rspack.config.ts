import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
let builds = 0;
let firstBuilds: number;
let firstAssets: string[];

export default defineConfig({
  entry: './src/index.js',
  mode: 'development',
  devtool: false,
  incremental: false,
  cache: { type: 'persistent' },
  experiments: {
    outputModule: true,
    newCache: { module: true, loader: false, codeGeneration: false },
  },
  module: { rules: [{ test: /\.png$/, type: 'asset/resource' }] },
  output: {
    module: true,
    filename: '[name].mjs',
    library: {
      type: 'modern-module',
      preserveModules: path.join(import.meta.dirname, 'src'),
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'SharedAssetModuleCache',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'SharedAssetModuleCache',
              () => builds++,
            );
          },
        );
        compiler.hooks.done.tap('SharedAssetModuleCache', (stats) => {
          expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
          const assets = stats.compilation
            .getAssets()
            .map((asset) => asset.name)
            .sort();
          expect(assets).toContain('index.png');
          expect(assets).toContain('components/button/index.png');
          if (buildIndex === 0) {
            firstBuilds = builds;
            firstAssets = assets;
            expect(firstBuilds).toBeGreaterThan(0);
          } else {
            expect(builds).toBe(firstBuilds);
            expect(assets).toEqual(firstAssets);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
