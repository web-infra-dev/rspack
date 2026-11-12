import { defineConfig } from '@rspack/cli';
import tsCheckerRspackPlugin from 'ts-checker-rspack-plugin';

const { TsCheckerRspackPlugin } = tsCheckerRspackPlugin;

export default defineConfig({
  mode: 'development',
  entry: {
    output: './index.ts',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
    ],
  },
  resolve: {
    extensions: ['.ts', '.js', '.json'],
  },
  plugins: [new TsCheckerRspackPlugin({ async: false })],
});
