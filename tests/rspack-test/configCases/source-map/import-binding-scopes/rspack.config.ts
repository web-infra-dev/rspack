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
  { devtool: 'source-map-scopes' },
  { devtool: 'cheap-module-source-map-scopes' },
] as const;

export default defineConfig(
  configs.map(({ devtool, ...config }, index) => ({
    devtool,
    target: 'node',
    mode: index === 5 ? 'production' : 'development',
    module: {
      rules: [
        {
          test: /\.ts$/,
          loader: 'builtin:swc-loader',
          options: { jsc: { parser: { syntax: 'typescript' } } },
        },
      ],
    },
    optimization: {
      concatenateModules: false,
      minimize: false,
      mangleExports: index === 5 ? 'deterministic' : false,
    },
    plugins: [
      ...('plugins' in config ? config.plugins : []),
      new DefinePlugin({ CONFIG_INDEX: index }),
    ],
  })),
);
