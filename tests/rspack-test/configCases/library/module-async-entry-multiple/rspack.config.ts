import { defineConfig } from '@rspack/cli';
import { container, type Configuration } from '@rspack/core';

const configurations: Configuration[] = [];

for (const target of ['node', 'web', 'webworker'] as const) {
  for (const startup of ['inline', 'split', 'runtime', 'startup-entrypoint']) {
    for (const lastEntry of ['last', 'last-async', 'single', 'single-no-tla']) {
      const name = `${target}-${startup}-${lastEntry}`;
      configurations.push({
        name,
        mode: 'development',
        devtool: false,
        target: [target, 'es2022'],
        entry: {
          main: './index.js',
          library: {
            import: lastEntry.startsWith('single')
              ? './single.js'
              : ['./first.js', `./${lastEntry}.js`],
            library: { type: 'module' },
          },
        },
        output: {
          module: true,
          filename: `${name}/[name].mjs`,
          chunkFilename: `${name}/[name].mjs`,
          environment: { topLevelAwait: lastEntry !== 'single-no-tla' },
        },
        externalsType: 'import',
        externals: { library: './library.mjs' },
        optimization: {
          runtimeChunk:
            startup === 'runtime' || startup === 'startup-entrypoint'
              ? 'single'
              : false,
          splitChunks: startup === 'split' && {
            cacheGroups: {
              state: {
                test: /state\.js$/,
                name: 'state',
                chunks: 'all',
                enforce: true,
              },
            },
          },
        },
        plugins:
          startup === 'startup-entrypoint'
            ? [
                new container.ModuleFederationPlugin({
                  name: 'async_entry_library',
                  experiments: { asyncStartup: true },
                }),
              ]
            : [],
      });
    }
  }
}

export default defineConfig(configurations);
