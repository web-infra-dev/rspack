import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  mode: 'development',
  entry: './index.js',
  output: {
    hashFunction: 'xxhash64',
    cssFilename: 'main.css',
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
        localIdentHashDigestLength: 16,
        localIdentHashFunction: 'xxhash64',
        localIdentName: '[hash]-[local]',
      },
    },
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
      {
        include: path.resolve(import.meta.dirname, 'legacy'),
        test: /\.css$/,
        type: 'css/module',
        parser: {
          namedExports: false,
        },
        generator: {
          exportsConvention: 'camel-case',
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 16,
          localIdentHashFunction: 'xxhash64',
          localIdentName: '[hash]-[local]',
        },
      },
    ],
  },
});
