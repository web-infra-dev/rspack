import { defineConfig } from '@rslib/core';
import packageJson from './package.json' with { type: 'json' };

export default defineConfig({
  lib: [
    {
      format: 'esm',
      syntax: ['es2023'],
      dts: {
        bundle: true,
        tsgo: true,
      },
    },
  ],
  output: {
    externals: [
      ({ request }, callback) => {
        if (request === 'jiti') {
          return callback(undefined, '../compiled/jiti/index.js');
        }
        return callback();
      },
    ],
  },
  source: {
    tsconfigPath: './tsconfig.build.json',
    define: {
      RSPACK_CLI_VERSION: JSON.stringify(packageJson.version),
    },
  },
});
