const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  entry: './index.ts',
  target: 'node',
  output: {
    library: {
      type: 'commonjs',
    },
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        type: 'javascript/auto',
        use: {
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
