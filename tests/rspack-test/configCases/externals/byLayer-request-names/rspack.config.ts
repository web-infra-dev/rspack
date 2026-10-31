import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((dynamic) => {
    const layers = {
      app: {},
      other: { bytes: { commonjs: 'other' }, bypass: { var: '45' } },
      default: { bytes: { var: '44' } },
    };
    return {
      externals: {
        bytes: { var: '42', commonjs: 'bytes' },
        byLayer: dynamic
          ? (layer) => layers[layer as keyof typeof layers] ?? layers.default
          : layers,
      },
      module: {
        rules: [
          { test: /app\.js$/, layer: 'app' },
          { test: /other\.js$/, layer: 'other' },
        ],
      },
    };
  }),
);
