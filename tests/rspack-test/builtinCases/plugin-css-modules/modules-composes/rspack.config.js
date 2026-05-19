/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        parser: {
          exportsOnly: false,
          namedExports: false,
        },
        generator: {
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 16,
          localIdentHashFunction: 'xxhash64',
          localIdentName: '[path]_[name]_[path]_[local]--/__[hash:42]<[hash:3]',
        },
      },
    ],
  },
};
