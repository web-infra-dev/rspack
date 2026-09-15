import { fileURLToPath } from 'node:url';
import { define } from 'rstack';

define.lib({
  lib: (['cjs', 'esm'] as const).map((format) => ({
    format,
    syntax: ['es2023'],
    bundle: false,
    shims: {
      esm: {
        require: true,
      },
    },
    redirect: {
      dts: {
        extension: true,
      },
    },
    dts: format === 'esm' && {
      tsgo: true,
      typescriptPath: fileURLToPath(import.meta.resolve('@typescript/native')),
    },
  })),
  source: {
    tsconfigPath: './tsconfig.build.json',
  },
});
