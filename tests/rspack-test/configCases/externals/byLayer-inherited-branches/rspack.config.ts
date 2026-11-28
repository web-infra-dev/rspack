import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].flatMap((dynamic) =>
    [{ app: { value: 'var 43' } }, { default: { value: 'var 44' } }].map(
      (inherited) => {
        const item = {
          value: 'var 42',
          byLayer: Object.create(inherited),
        };
        return {
          externals: dynamic ? { byLayer: () => item } : item,
          module: { rules: [{ test: /app\.js$/, layer: 'app' }] },
        };
      },
    ),
  ),
);
