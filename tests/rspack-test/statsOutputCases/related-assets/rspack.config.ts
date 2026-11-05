import { defineConfig } from '@rspack/cli';
import type { Configuration, Compiler } from '@rspack/core';
import { Compilation, CssExtractRspackPlugin } from '@rspack/core';

const compression = (exts: string[]) => (compiler: Compiler) => {
  compiler.hooks.thisCompilation.tap('Test', (compilation) => {
    compilation.hooks.processAssets.tap(
      {
        name: 'Test',
        stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER,
      },
      () => {
        for (const asset of compilation.getAssets()) {
          for (const ext of exts) {
            const newFile = `${asset.name}${ext}`;
            compilation.emitAsset(newFile, asset.source);
            compilation.updateAsset(asset.name, asset.source, {
              related: {
                // @ts-expect-error Custom related asset keys are not exposed in Rspack types yet.
                compressed: ['...', newFile],
              },
            });
          }
        }
      },
    );
  });
};

const base = (name: string): Configuration => ({
  name,
  mode: 'development',
  devtool: 'source-map',
  entry: './index',
  output: {
    filename: `${name}-[name].js`,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        use: [
          CssExtractRspackPlugin.loader,
          {
            loader: 'css-loader',
            options: {
              sourceMap: true,
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: `${name}-[name].css`,
    }),
    compression(['.br', '.gz']),
  ],
});

const baseStats = {
  assets: true,
  entrypoints: false,
  modules: false,
  timings: false,
  version: false,
  hash: false,
  builtAt: false,
  errorsCount: false,
  warningsCount: false,
};

export default defineConfig([
  {
    ...base('default'),
    stats: {
      ...baseStats,
    },
  },
  {
    ...base('relatedAssets'),
    stats: {
      ...baseStats,
      relatedAssets: true,
    },
  },
  {
    ...base('exclude1'),
    stats: {
      ...baseStats,
      relatedAssets: true,
      excludeAssets: /\.(gz|br)$/,
    },
  },
  {
    ...base('exclude2'),
    stats: {
      ...baseStats,
      relatedAssets: true,
      excludeAssets: /\.map$/,
    },
  },
  {
    ...base('exclude3'),
    stats: {
      ...baseStats,
      relatedAssets: true,
      excludeAssets: /chunk/,
    },
  },
]);
