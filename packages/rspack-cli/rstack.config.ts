import { fileURLToPath } from 'node:url';
import { define } from 'rstack';
import packageJson from './package.json' with { type: 'json' };

define.lib({
  lib: [
    {
      format: 'esm',
      syntax: ['es2023'],
      dts: {
        bundle: true,
        tsgo: true,
        typescriptPath: fileURLToPath(
          import.meta.resolve('@typescript/native'),
        ),
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
