import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  [false, true].map((rslib) => {
    const name = rslib ? 'rslib' : 'plain';
    return defineConfig({
      name,
      target: 'node',
      entry: rslib ? './index.js' : './plain.mjs',
      externalsType: 'module',
      externals: ['./ext.mjs', './other.mjs', './side.mjs'],
      output: {
        filename: `${name}/[name].mjs`,
        chunkFilename: `${name}/[name].mjs`,
      },
      plugins: rslib ? [new rspack.experiments.RslibPlugin()] : [],
      optimization: {
        minimize: false,
        runtimeChunk: false,
      },
    });
  }),
);
