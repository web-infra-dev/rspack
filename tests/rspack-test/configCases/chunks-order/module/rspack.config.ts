import { defineConfig } from '@rspack/cli';

export default defineConfig((_env, { testPath: _testPath }) => [
  {
    target: 'node',
    entry: {
      'nested-shared': {
        import: './nested-shared.js',
      },
      shared: {
        dependOn: 'nested-shared',
        import: './shared.js?foo=bar',
      },
      foo: {
        dependOn: 'shared',
        import: './foo.js',
      },
    },
    output: {
      publicPath: 'auto',
      filename: '[name].mjs',
      chunkFilename: '[name].[contenthash].mjs',
      cssChunkFilename: '[name].[contenthash].css',
      assetModuleFilename: '[name].[contenthash][ext][query]',
      library: {
        type: 'module',
      },
    },
    devtool: 'source-map',
    module: {
      rules: [
        {
          test: /\.txt$/,
          type: 'asset/resource',
        },
        {
          test: /\.css$/,
          type: 'css',
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
    experiments: {
      css: true,
      outputModule: true,
    },
    optimization: {
      chunkIds: 'named',
      splitChunks: {
        cacheGroups: {
          commons: {
            enforce: true,
            test: /dependency\.js$/,
            chunks: 'initial',
          },
        },
      },
      runtimeChunk: { name: (entrypoint) => `runtime~${entrypoint.name}` },
    },
  },
]);
