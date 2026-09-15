const path = require('path');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  output: {
    hashFunction: 'md4',
    hashDigestLength: 20,
  },
  module: {
    parser: {
      'css/auto': {
        namedExports: true,
      },
    },
    generator: {
      'css/auto': {
        exportsConvention: 'as-is',
        localIdentHashDigest: 'hex',
        localIdentHashDigestLength: 20,
        localIdentHashFunction: 'md4',
        localIdentName: '[hash]-[local]',
        exportsOnly: true,
      },
    },
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
      {
        include: path.resolve(__dirname, 'legacy'),
        test: /\.css$/,
        type: 'css/module',
        parser: {
          namedExports: false,
        },
        generator: {
          exportsConvention: 'camel-case',
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 20,
          localIdentHashFunction: 'md4',
          localIdentName: '[hash]-[local]',
        },
      },
    ],
  },
};
