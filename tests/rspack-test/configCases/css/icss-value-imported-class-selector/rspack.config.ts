import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const variants = [
  {
    name: 'development',
    mode: 'development' as const,
    minimize: false,
    localIdentName: '[name]__[local]',
  },
  {
    name: 'production',
    mode: 'production' as const,
    minimize: false,
    localIdentName: '[name]__[local]',
  },
  {
    name: 'production-minified-local',
    mode: 'production' as const,
    minimize: true,
    localIdentName: '[local]',
  },
];

export default defineConfig(
  variants.map(({ name, mode, minimize, localIdentName }, idx) =>
    defineConfig({
      name,
      mode,
      devtool: false,
      entry: './index.js',
      target: 'web',
      externals: {
        fs: 'node-commonjs fs',
        path: 'node-commonjs path',
      },
      output: {
        filename: `bundle${idx}.js`,
        cssFilename: `bundle${idx}.css`,
      },
      node: {
        __dirname: false,
        __filename: false,
      },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'css/auto',
            parser: {
              dashedIdents: true,
            },
            generator: {
              localIdentName,
            },
          },
        ],
      },
      optimization: {
        minimize,
        minimizer: minimize
          ? [new rspack.LightningCssMinimizerRspackPlugin()]
          : [],
      },
      plugins: [
        new rspack.DefinePlugin({
          'process.env.VARIANT': JSON.stringify(name),
        }),
      ],
      experiments: {
        css: true,
      },
    }),
  ),
);
