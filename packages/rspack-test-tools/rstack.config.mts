import { fileURLToPath } from 'node:url';
import { define } from 'rstack';

define.lib({
  lib: [
    {
      format: 'cjs',
      syntax: ['es2023'],
      bundle: false,
      dts: {
        tsgo: true,
        typescriptPath: fileURLToPath(
          import.meta.resolve('@typescript/native'),
        ),
      },
    },
  ],
  source: {
    tsconfigPath: './tsconfig.build.json',
  },
});
