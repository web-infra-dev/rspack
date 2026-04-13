import path from 'node:path';
import { defineConfig } from '@rsbuild/core';

export default defineConfig({
  source: {
    entry: {
      'basic-react': './cases/basic-react/index.js',
    },
  },
  html: {
    template({ entryName }) {
      return `./cases/${entryName}/index.html`;
    },
  },
  output: {
    target: 'web',
  },
  resolve: {
    alias: {
      '@rspack/browser': path.resolve(
        __dirname,
        '../../packages/rspack-browser/dist/index.js',
      ),
    },
  },
  server: {
    port: 8900,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
