import { defineConfig } from '@rspack/cli';

import { fileURLToPath } from 'node:url';

export default defineConfig([
  {
    name: fileURLToPath(import.meta.resolve('./app.js')),
    mode: 'production',
    entry: './app.js',
    output: {
      filename: 'bundle1.js',
    },
    stats: {
      assets: true,
      modules: true,
    },
  },
  {
    name: fileURLToPath(import.meta.resolve('./server.js')),
    mode: 'production',
    entry: './server.js',
    output: {
      filename: 'bundle2.js',
    },
    stats: {
      assets: true,
      modules: true,
    },
  },
]);
