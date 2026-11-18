import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

export default defineConfig({
  optimization: { minimize: false },
  plugins: [
    new sharing.ConsumeSharedPlugin({
      shareScope: 'default',
      consumes: {
        'shared-async': {
          import: './fallback.js',
          shareKey: 'shared-async',
          requiredVersion: false,
          eager: false,
        },
        'shared-eager': {
          import: './fallback.js',
          shareKey: 'shared-eager',
          requiredVersion: false,
          eager: true,
        },
      },
    }),
  ],
});
