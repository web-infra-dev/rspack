import { defineConfig } from '@rslib/core';

export default defineConfig({
  lib: [
    {
      format: 'cjs',
      syntax: ['es2023'],
      bundle: false,
      dts: {
        tsgo: true,
      },
    },
  ],
  source: {
    tsconfigPath: './tsconfig.build.json',
  },
});
