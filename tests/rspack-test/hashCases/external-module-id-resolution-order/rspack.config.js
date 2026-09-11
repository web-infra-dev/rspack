const path = require('path');

function config(name, delayedRequest) {
  return {
    name,
    mode: 'production',
    context: __dirname,
    entry: './index.js',
    target: 'node',
    cache: false,
    devtool: 'source-map',
    output: {
      path: path.resolve(__dirname, `dist/${name}`),
      filename: 'bundle.js',
      library: { type: 'commonjs2' },
    },
    optimization: {
      moduleIds: 'named',
      minimize: false,
      concatenateModules: false,
    },
    externals: [
      ({ request }, callback) => {
        if (request !== './shared' && request !== '../shared') {
          return callback();
        }
        setTimeout(
          () => callback(null, 'commonjs node:path'),
          request === delayedRequest ? 100 : 0,
        );
      },
    ],
  };
}

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  config('delay-parent-request', '../shared'),
  config('delay-current-request', './shared'),
];
