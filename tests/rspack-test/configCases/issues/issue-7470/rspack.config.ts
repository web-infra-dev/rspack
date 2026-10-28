import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig([
  {
    name: 'development',
    mode: 'development',
    plugins: [new DefinePlugin({ __MODE__: `"development"` })],
  },
  {
    name: 'production',
    mode: 'production',
    plugins: [new DefinePlugin({ __MODE__: `"production"` })],
  },
  {
    name: 'none',
    mode: 'none',
    plugins: [new DefinePlugin({ __MODE__: `"none"` })],
  },
]);
