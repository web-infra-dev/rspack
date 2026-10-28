import path from 'node:path';
import { defineConfig } from '@rspack/cli';

function config(name: string, delayedRequest: string) {
  return defineConfig({
    name,
    mode: 'production',
    context: import.meta.dirname,
    entry: './index.js',
    target: 'node',
    cache: false,
    devtool: 'source-map',
    output: {
      path: path.resolve(import.meta.dirname, `dist/${name}`),
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
          () => callback(undefined, 'commonjs node:path'),
          request === delayedRequest ? 100 : 0,
        );
      },
    ],
  });
}

export default defineConfig([
  config('delay-parent-request', '../shared'),
  config('delay-current-request', './shared'),
]);
