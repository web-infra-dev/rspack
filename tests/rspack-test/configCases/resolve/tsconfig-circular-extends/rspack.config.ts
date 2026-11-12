import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default [
  'self.json',
  'a.json',
  'array.json',
  'references.json',
  'diamond.json',
].map((configFile) =>
  defineConfig({
    resolve: {
      tsConfig: {
        configFile: path.resolve(import.meta.dirname, configFile),
        references: 'auto',
      },
    },
  }),
);
