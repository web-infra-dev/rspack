/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: {
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 16,
          localIdentHashFunction: 'xxhash64',
          localIdentName: '[hash]',
        },
      },
    ],
  },
};
