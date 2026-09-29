import { defineConfig } from '@rspack/cli';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  plugins: [
    // This plugin should only affect contexts matching /components$/
    // and should NOT affect the "assets" context
    new webpack.ContextReplacementPlugin(
      /components$/,
      './replaced-components',
      true,
      /^replaced$/,
    ),
  ],
});
