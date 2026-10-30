import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

const { RawSource } = rspack.sources;

class CopyPlugin {
  apply(compiler: Compiler) {
    const hookOptions = {
      name: 'MockCopyPlugin',
      stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
    };

    compiler.hooks.thisCompilation.tap(hookOptions, (compilation) => {
      compilation.hooks.processAssets.tap(hookOptions, () => {
        const output = '// some compilation result\n';
        compilation.emitAsset('third.party.js', new RawSource(output));
      });
    });
  }
}

export default defineConfig([
  {
    node: {
      __dirname: false,
      __filename: false,
    },
    output: {
      publicPath: '/app/',
      chunkFilename: '[name].[contenthash].js',
      assetModuleFilename: '[name].[contenthash][ext][query]',
    },
    plugins: [
      new CopyPlugin(),
      // @ts-expect-error Rspack does not expose ManifestPlugin yet.
      new rspack.ManifestPlugin({
        filename: 'foo.json',
      }),
    ],
    module: {
      rules: [
        {
          test: /\.txt$/,
          type: 'asset/resource',
        },
        {
          test: /\.png$/,
          loader: 'file-loader',
          options: {
            name: 'file-loader.[ext]',
          },
        },
      ],
    },
  },
  {
    entry: './index-2.js',
    node: {
      __dirname: false,
      __filename: false,
    },
    output: {
      publicPath: (_data) => '/dist/',
      chunkFilename: '[name].[contenthash].js',
      assetModuleFilename: '[name].[contenthash][ext][query]',
    },
    plugins: [
      new CopyPlugin(),
      // @ts-expect-error Rspack does not expose ManifestPlugin yet.
      new rspack.ManifestPlugin({
        filename: 'bar.json',
      }),
    ],
    module: {
      rules: [
        {
          test: /\.txt$/,
          type: 'asset/resource',
        },
        {
          test: /\.png$/,
          loader: 'file-loader',
          options: {
            name: 'file-loader.[ext]',
          },
        },
      ],
    },
  },
]);
