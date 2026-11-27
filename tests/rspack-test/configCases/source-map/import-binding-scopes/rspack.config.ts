import { fileURLToPath } from 'node:url';
import {
  BannerPlugin,
  DefinePlugin,
  SourceMapDevToolPlugin,
} from '@rspack/core';
import { defineConfig } from '@rspack/cli';

const configs = [
  {
    devtool: 'source-map-scopes',
    plugins: [
      new BannerPlugin({
        raw: true,
        footer: true,
        banner:
          '{ const onlyFooter = 60; globalThis.scopeFooterValue = onlyFooter; debugger; }',
      }),
    ],
  },
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
        {
          test: /jsx-entry\.js$/,
          loader: 'builtin:swc-loader',
          options: {
            jsc: {
              parser: { syntax: 'ecmascript', jsx: true },
              transform: { react: { pragma: 'h' } },
            },
          },
        },
        {
          test: /shared-variant\.js$/,
          loader: fileURLToPath(
            new URL('./shared-source-loader.cjs', import.meta.url),
          ),
        },
        {
          test: /wrapped-entry\.js$/,
          loader: fileURLToPath(new URL('./wrap-loader.cjs', import.meta.url)),
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
