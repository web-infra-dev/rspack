import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.ContextReplacementPlugin(
      /replacement.a$/,
      'new-context',
      true,
      /^replaced$/,
    ),
  ],
});
