import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((dynamic) => {
    const app = {
      disabled: { var: '41' },
      scalar: { var: '43' },
      array: ['var ({ value: 45 })', 'value'],
      toArray: ['var ({ value: 46 })', 'value'],
      extended: ['...', 'value'],
      object: { commonjs: 'app' },
    };
    return {
      externalsPresets: { node: false },
      externals: {
        disabled: false,
        scalar: 'var 42',
        array: ['var ({ other: 44 })', 'other'],
        toArray: 'var 42',
        extended: ['var ({ nested: { value: 47 } })', 'nested'],
        object: { var: '42', commonjs: 'base' },
        byLayer: dynamic ? () => app : { app },
      },
      module: { rules: [{ test: /app\.js$/, layer: 'app' }] },
    };
  }),
);
