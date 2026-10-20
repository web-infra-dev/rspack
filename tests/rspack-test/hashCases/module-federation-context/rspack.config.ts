import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { type Configuration, container } from '@rspack/core';

export default defineConfig(
  [false, true].flatMap((enhanced) =>
    ['a', 'b'].map((directory): Configuration => {
      const context = path.resolve(import.meta.dirname, directory);
      const ModuleFederationPlugin = enhanced
        ? container.ModuleFederationPlugin
        : container.ModuleFederationPluginV1;

      return {
        mode: 'production',
        context,
        entry: './index.js',
        devtool: false,
        output: {
          path: path.resolve(
            import.meta.dirname,
            `dist/${enhanced ? 'enhanced' : 'v1'}/${directory}`,
          ),
          filename: '[name].[contenthash].js',
          chunkFilename: '[id].[contenthash].js',
        },
        optimization: {
          moduleIds: 'deterministic',
          chunkIds: 'deterministic',
          minimize: false,
        },
        plugins: [
          new ModuleFederationPlugin({
            name: 'app',
            filename: 'remoteEntry.js',
            exposes: { './lazy': './lazy.js' },
            shared: {
              'shared-lib': {
                import: './shared.js',
                version: '1.0.0',
                requiredVersion: '^1.0.0',
                singleton: true,
              },
            },
            ...(enhanced
              ? { runtimePlugins: [path.resolve(context, 'runtime-plugin.js')] }
              : {}),
          }),
        ],
      };
    }),
  ),
);
