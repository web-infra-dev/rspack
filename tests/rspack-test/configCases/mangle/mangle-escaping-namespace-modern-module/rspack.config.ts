import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [undefined, path.resolve(import.meta.dirname, '..')].map(
    (preserveModules, index) => ({
      mode: 'production',
      target: 'node',
      entry: {
        main: { import: './index.js', filename: `bundle${index}.mjs` },
      },
      output: {
        filename: `${index}-[name].mjs`,
        library: { type: 'modern-module', preserveModules },
      },
      optimization: { minimize: false, mangleExports: true },
    }),
  ),
);
