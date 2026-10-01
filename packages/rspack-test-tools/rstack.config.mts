import { fileURLToPath } from 'node:url';
import { define } from 'rstack';

const commonConfig = {
  syntax: ['es2023'],
  bundle: false,
};

define.lib({
  tools: {
    rspack: {
      experiments: {
        runtimeMode: 'rspack',
      },
    },
  },
  lib: [
    {
      ...commonConfig,
      format: 'cjs',
    },
    {
      ...commonConfig,
      format: 'esm',
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
