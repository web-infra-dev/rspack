import { createRequire } from 'node:module';
import { defineConfig } from '@rslib/core';

const require = createRequire(import.meta.url);

export default defineConfig({
  lib: [
    {
      format: 'cjs',
      syntax: ['es2023'],
      dts: true,
    },
    {
      format: 'esm',
      syntax: ['es2023'],
    },
  ],
  source: {
    tsconfigPath: './tsconfig.build.json',
    define: {
      RSPACK_CLI_VERSION: JSON.stringify(require('./package.json').version),
    },
  },
});
