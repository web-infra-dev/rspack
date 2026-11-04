import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [true, false].map((mangleExports) => ({
    mode: 'production',
    optimization: {
      minimize: false,
      moduleIds: 'named',
      mangleExports,
    },
  })),
);
