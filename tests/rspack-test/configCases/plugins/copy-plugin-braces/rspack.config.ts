import { defineConfig } from '@rspack/cli';
import { CopyRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: 'src/{foo,bar}.*.yml',
          to: 'wildcard/[name][ext]',
        },
        {
          from: 'src/{alpha,beta}.txt',
          to: 'literal/[name][ext]',
        },
        {
          from: 'src/{one,two}/**/*.txt',
          to: 'nested/[path][name][ext]',
        },
      ],
    }),
  ],
  output: {
    clean: true,
  },
});
