const os = require('node:os');
const path = require('node:path');

const cacheDirectory = path.join(
  os.tmpdir(),
  `rspack-builtin-cache-loader-${process.pid}`,
);

const createConfig = (name, filename, dependencies) => ({
  name,
  dependencies,
  entry: './index.js',
  output: { filename },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          {
            loader: 'builtin:cache-loader',
            /** @type {import('@rspack/core').CacheLoaderOptions} */
            options: {
              cacheDirectory,
              cacheIdentifier: 'basic',
            },
          },
          path.resolve(__dirname, 'count-loader.js'),
        ],
      },
    ],
  },
});

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  createConfig('prime-cache', 'bundle0.js'),
  createConfig('reuse-cache', 'bundle1.js', ['prime-cache']),
];
