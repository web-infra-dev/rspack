import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { ProvideSharedPlugin, ConsumeSharedPlugin } = sharing;

export default defineConfig({
  mode: 'development',
  devtool: false,
  ignoreWarnings: [/No version specified/],
  plugins: [
    new ProvideSharedPlugin({
      provides: ['no-version-pkg'],
    }),
    new ConsumeSharedPlugin({
      consumes: {
        'no-version-pkg': {
          requiredVersion: '^1.0.0',
          strictVersion: true,
          singleton: true,
        },
      },
    }),
  ],
});
