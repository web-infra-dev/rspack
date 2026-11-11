import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

export default defineConfig(
  [
    ['tsconfig.json', 'src'],
    ['reversed.json', 'src'],
    ['own-paths.json', 'src'],
    ['own-base-url.json', 'own'],
    ['no-base-url.json', 'config'],
    ['template-base-url.json', 'src'],
  ].map(([filename, expected]) => ({
    resolve: {
      tsConfig: path.resolve(import.meta.dirname, filename),
    },
    plugins: [new rspack.DefinePlugin({ EXPECTED: JSON.stringify(expected) })],
  })),
);
