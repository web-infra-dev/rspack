import { defineConfig } from '@rspack/cli';
import { type Configuration } from '@rspack/core';

const createConfig = (
  name: string,
  output: Configuration['output'],
): Configuration => ({
  name,
  mode: 'none',
  output,
  module: {
    parser: {
      javascript: {
        worker: {
          url: 'new-url-relative',
        },
      },
    },
  },
});

export default defineConfig([
  createConfig('non-esm', {
    filename: 'non-esm.js',
    chunkFilename: 'non-esm-[name].bundle.js',
    publicPath: '/public/',
  }),
  createConfig('public-path', {
    module: true,
    filename: 'public-path.js',
    chunkFilename: 'public-path-[name].bundle.js',
    publicPath: '/public/',
  }),
  createConfig('relative-public-path', {
    module: true,
    filename: 'relative-public-path/main.js',
    chunkFilename: 'relative-public-path-[name].bundle.js',
    publicPath: 'assets/',
  }),
  createConfig('worker-public-path', {
    module: true,
    filename: 'worker-public-path.js',
    chunkFilename: 'worker-public-path-[name].bundle.js',
    publicPath: '/public/',
    workerPublicPath: '/workers/',
  }),
  createConfig('relative-worker-public-path', {
    module: true,
    filename: 'relative-worker-public-path/main.js',
    chunkFilename: 'relative-worker-public-path-[name].bundle.js',
    publicPath: '/public/',
    workerPublicPath: 'workers/',
  }),
]);
