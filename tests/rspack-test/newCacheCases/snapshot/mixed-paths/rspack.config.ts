import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
      managedPaths: [/^(.+?[\\/]packages[\\/])/],
    },
  },
  snapshot: {
    unmanagedPaths: [path.join(import.meta.dirname, 'immutable/mutable')],
  },
  module: {
    rules: [{ test: /consumer-.*\.js$/, loader: './dependency-loader.js' }],
  },
});
