import { defineConfig } from '@rspack/cli';
import { container, DefinePlugin } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig(
  [
    { shareKey: 'shared (custom)', shareScope: 'default', version: '1.2.3' },
    { shareKey: '@scope/shared', shareScope: 'team = beta', version: '1.2.3' },
    { shareKey: 'shared', shareScope: 'default', version: false as const },
  ].map(({ shareKey, shareScope, version }, index) => ({
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    output: {
      uniqueName: `manifest-required-version-range-${index}`,
      chunkFilename: `${index}-[id].js`,
    },
    plugins: [
      new DefinePlugin({
        CASE_INDEX: index,
        CONSUME_SHARED: index === 0,
        SHARE_KEY: JSON.stringify(shareKey),
        SHARED_VERSION: JSON.stringify(version === false ? '0' : version),
      }),
      new ModuleFederationPlugin({
        name: `container${index}`,
        filename: `${index}-container.js`,
        library: { type: 'commonjs-module' },
        shareScope,
        manifest: { fileName: `mf-${index}.json` },
        exposes: { './module': './module.js' },
        shared: {
          // Trim outer whitespace in the inferred range, preserving spaces around ||.
          xreact: {},
          // Configured, overriding the "^2.0.0" in package.json.
          yreact: { requiredVersion: '^2.0.0 || ^3.0.0' },
          // Exercise providers without a consumer in the scope and false cases.
          [index === 0 ? './shared.js' : 'unused-consumer']: {
            import: './shared.js',
            shareKey,
            version,
            requiredVersion: false,
          },
        },
      }),
    ],
  })),
);
