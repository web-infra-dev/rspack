import { defineConfig } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

const configurations: Configuration[] = [];

for (const mode of ['development', 'production'] as const) {
  for (const startup of [
    'inline',
    'inline-no-format',
    'runtime',
    'runtime-no-tla',
  ]) {
    const name = `${mode}-${startup}`;
    configurations.push({
      name,
      mode,
      target: ['node', 'es2022'],
      devtool: false,
      entry: {
        main: './index.js',
        library: {
          import: './entry.js',
          library: { type: 'module' },
        },
      },
      output: {
        module: true,
        chunkFormat: startup === 'inline-no-format' ? false : undefined,
        filename: `${name}/[name].mjs`,
        chunkFilename: `${name}/[name].mjs`,
        environment: { topLevelAwait: startup !== 'runtime-no-tla' },
      },
      externalsType: 'module',
      externals: { library: './library.mjs' },
      optimization: {
        runtimeChunk: startup.startsWith('runtime') ? 'single' : false,
      },
    });
  }
}

for (const chunkFormat of ['commonjs', 'array-push']) {
  const name = `chunk-format-${chunkFormat}`;
  configurations.push({
    name,
    target: ['node', 'es2022'],
    entry: { library: './entry.js' },
    output: {
      module: true,
      chunkFormat,
      library: { type: 'module' },
      filename: `${name}/[name].mjs`,
      environment: { topLevelAwait: true },
    },
    optimization: { runtimeChunk: 'single' },
  });
}

export default defineConfig(configurations);
