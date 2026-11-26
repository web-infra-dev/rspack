import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;

export default defineConfig({
  mode: 'development',
  devtool: false,
  ignoreWarnings: [/No version specified/],
  plugins: [
    new ProvideSharedPlugin({
      provides: ['no-version-pkg'],
    }),
  ],
});
