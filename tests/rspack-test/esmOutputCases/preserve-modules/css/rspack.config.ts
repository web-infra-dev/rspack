import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  target: 'web',
  entry: './src/index.js',
  experiments: {
    css: true,
  },
  module: {
    generator: {
      'css/auto': {
        // force the css module to emit a `.css` asset (not inline as JS)
        // so that preserveModules has to handle a real CSS chunk.
        exportsOnly: false,
      },
    },
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
});
