import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

export default defineConfig({
  mode: 'development',
  devtool: false,
  plugins: [
    new SharePlugin({
      shared: {
        common: {
          eager: true,
          import: './common?1',
          requiredVersion: '1.1.1',
        },
        common2: {
          import: './common?2',
          requiredVersion: '1.1.1',
        },
      },
    }),
  ],
});
