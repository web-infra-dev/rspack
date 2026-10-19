import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

export default defineConfig({
  plugins: [
    new sharing.ConsumeSharedPlugin({
      consumes: {
        strict: {
          import: false,
          shareKey: 'shared',
          requiredVersion: '^1.0.0',
          strictVersion: true,
          singleton: false,
          eager: true,
        },
        singleton: {
          import: false,
          shareKey: 'shared',
          requiredVersion: '^1.0.0',
          strictVersion: false,
          singleton: true,
          eager: true,
        },
      },
    }),
  ],
});
