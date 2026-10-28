import { defineConfig } from '@rspack/cli';
import { type Compiler, type Configuration, NormalModule } from '@rspack/core';
import path from 'node:path';

const hashes = new Set();

class CheckHashPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(CheckHashPlugin.name, (compilation) => {
      compilation.hooks.processAssets.tap(CheckHashPlugin.name, () => {
        const barrelModule = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            module.resource?.endsWith('barrel.js'),
        )!;
        expect(barrelModule).toBeTruthy();

        const moduleHash = compilation.chunkGraph.getModuleHash(
          barrelModule,
          'main',
        );
        expect(moduleHash).toBeTruthy();

        hashes.add(moduleHash);
        expect(hashes.size).toBe(1);
      });
    });
  }
}

function config(name: string): Configuration {
  return {
    mode: 'production',
    entry: './barrel.js',
    devtool: false,
    output: {
      path: path.join(import.meta.dirname, 'dist', name),
      filename: '[name].[contenthash].js',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
      minimize: false,
      realContentHash: false,
    },
    plugins: [new CheckHashPlugin()],
  };
}

export default defineConfig(
  Array.from({ length: 4 }, (_, i) => config(String(i))),
);
