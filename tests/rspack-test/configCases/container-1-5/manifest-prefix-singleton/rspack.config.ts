import { defineConfig } from '@rspack/cli';
import { container, DefinePlugin } from '@rspack/core';

export default defineConfig(
  [false, true].map((consumeOnly, index) => {
    const defaultSharedOptions = {
      requiredVersion: '^1.0.0',
      ...(consumeOnly ? { import: false as const } : {}),
    };
    const sharedOptions = { ...defaultSharedOptions, singleton: false };

    return {
      externals: {
        fs: 'node-commonjs fs',
        path: 'node-commonjs path',
      },
      output: {
        uniqueName: `manifest-prefix-singleton-${index}`,
        chunkFilename: `${index}-[id].js`,
      },
      plugins: [
        new DefinePlugin({
          CASE_INDEX: index,
          CONSUME_ONLY: consumeOnly,
        }),
        new container.ModuleFederationPlugin({
          name: `container${index}`,
          filename: `${index}-container.js`,
          library: { type: 'commonjs-module' },
          shareScope: `manifest-prefix-singleton-${index}`,
          manifest: { fileName: `mf-${index}.json` },
          exposes: { './module': './module.js' },
          shared: {
            'ds/': sharedOptions,
            'ds/Exact': { ...sharedOptions, singleton: true },
            'ds/Default': defaultSharedOptions,
            'ds/nested/': { ...sharedOptions, singleton: true },
            '@scope/lib/': { ...sharedOptions, shareKey: 'renamed/' },
            '@default/lib/': defaultSharedOptions,
          },
        }),
      ],
    };
  }),
);
