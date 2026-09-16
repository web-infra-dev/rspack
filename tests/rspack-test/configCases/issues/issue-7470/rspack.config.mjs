import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration[]} */
export default [
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
];
