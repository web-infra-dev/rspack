import { DefinePlugin, SourceMapDevToolPlugin } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

const configs = [
  { devtool: 'source-map-scopes' },
  { devtool: 'inline-nosources-source-map-scopes' },
  { devtool: 'eval-source-map-scopes' },
  {
    devtool: false,
    plugins: [
      new SourceMapDevToolPlugin({
        filename: '[file].map',
        scopes: true,
      }),
    ],
  },
  { devtool: 'source-map' },
] as const;

export default defineConfig(
  configs.map(({ devtool, ...config }, index) => ({
    devtool,
    target: 'node',
    mode: 'development',
    optimization: {
      concatenateModules: false,
      minimize: false,
    },
    plugins: [
      ...('plugins' in config ? config.plugins : []),
      new DefinePlugin({ CONFIG_INDEX: index }),
    ],
  })),
);
