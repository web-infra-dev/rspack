const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
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
        test: /\.[cm]?ts$/,
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
