import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: './loader.mjs!./index.js',
    mode: 'production',
    stats: {
      assets: true,
      modules: true,
      errorsSpace: 0,
      errors: true,
    },
  },
  {
    entry: './loader.mjs!./index.js',
    mode: 'production',
    stats: {
      assets: true,
      modules: true,
      errorsSpace: 2, // 2 errors (2 errors without details)
      errors: true,
    },
  },
  {
    entry: './loader.mjs!./index.js',
    mode: 'production',
    stats: {
      assets: true,
      modules: true,
      errorsSpace: 3, // 2 errors (2 errors without details)
      errors: true,
    },
  },
  {
    entry: './loader.mjs!./index.js',
    mode: 'production',
    stats: {
      assets: true,
      modules: true,
      errorsSpace: 4, // 2 errors + 2 lines (2 errors, one with partial details)
      errors: true,
    },
  },
  {
    entry: './loader.mjs!./index.js',
    mode: 'production',
    stats: {
      assets: true,
      modules: true,
      errorsSpace: 5, // 2 errors + 3 lines (2 errors, one full details)
      errors: true,
    },
  },
  {
    entry: './loader.mjs!./index.js',
    mode: 'production',
    stats: {
      assets: true,
      modules: true,
      errorsSpace: 100,
      errors: true,
    },
  },
]);
