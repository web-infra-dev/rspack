import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration[]} */
export default [
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
];
