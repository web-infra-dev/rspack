const path = require('node:path');

function config(name, sandbox, changeCode = false) {
  return {
    name,
    mode: 'production',
    devtool: 'hidden-source-map',
    entry: './index.js',
    output: {
      path: path.resolve(__dirname, 'dist', name),
      filename: '[name].[contenthash].js',
    },
    module: {
      rules: [
        {
          test: /index\.js$/,
          use: {
            loader: path.resolve(__dirname, 'loader.js'),
            // Keep module identity stable while varying the loader's source map.
            ident: 'source-map-path-independence',
            options: { sandbox, changeCode },
          },
        },
      ],
    },
    optimization: {
      minimize: false,
      // Hashing final asset bytes would hide source-map contributions to codegen hashes.
      realContentHash: false,
    },
  };
}

module.exports = [
  config('worker-0', '/mnt/engflow/worker/work/0/exec'),
  config('worker-7', '/mnt/engflow/worker/work/7/exec'),
  config('changed-code', '/mnt/engflow/worker/work/7/exec', true),
];
