const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: './index',
  stats: 'errors-warnings',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
          jsc: {
            experimental: {
              emitIsolatedDts: true,
            },
          },
        },
      },
    ],
  },
  plugins: [
    new RslibPlugin({
      emitDts: {
        rootDir: __dirname,
        declarationDir: './dist/types',
      },
    }),
  ],
};
