import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

export default defineConfig(
  [
    ['tsconfig.json', 'second'],
    ['reversed.json', 'first'],
    ['own-paths.json', 'own'],
    ['nested.json', 'second'],
    ['missing-paths.json', 'first'],
    ['base-url.json', 'second'],
    ['own-base-url.json', 'own'],
  ].map(([filename, expected]) => ({
    resolve: {
      tsConfig: path.resolve(import.meta.dirname, filename),
    },
    plugins: [new rspack.DefinePlugin({ EXPECTED: JSON.stringify(expected) })],
  })),
);
