import { defineConfig } from '@rspack/cli';
import { CopyRspackPlugin } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  target: 'node',
  entry: {
    main: './index.js',
  },
  resolve: {
    pnp: true,
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: '.yarn/cache/the-answer-npm-1.0.0-c6ae1caf6d-5870a78be9.zip/node_modules/the-answer/package.json',
          to: 'copied-package.json',
          preserveTimestamps: true,
        },
        {
          from: 'permissions.zip/executable.txt',
          to: 'copied-executable.txt',
          preservePermissions: true,
        },
      ],
    }),
  ],
});
