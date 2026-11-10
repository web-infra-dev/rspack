import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig(
  [
    './tsconfig.json',
    './tsconfig-extends.json',
    './tsconfig-references.json',
    './tsconfig-empty.json',
  ].map((filename) => ({
    resolve: {
      tsConfig: {
        configFile: path.resolve(import.meta.dirname, filename),
        references: 'auto',
      },
      alias:
        filename === './tsconfig-empty.json'
          ? { 'bom-value': path.resolve(import.meta.dirname, './value.js') }
          : undefined,
    },
  })),
);
