import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: './src/main.tsx',
  target: ['browserslist:last 2 versions, > 0.2%, not dead, Firefox ESR'],
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
    conditionNames: ['solid', '...'],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
      {
        test: /\.[cm]?[jt]sx?$/,
        resolve: {
          fullySpecified: false,
        },
        use: [
          {
            loader: 'builtin:swc-loader',
            options: { detectSyntax: 'auto' },
          },
          'solid-loader',
        ],
      },
      {
        // Solid loads hydration modules from URLs at runtime.
        descriptionData: { name: /^@solidjs\/web$/ },
        parser: { importDynamic: false },
      },
    ],
  },
  plugins: [new rspack.HtmlRspackPlugin({ template: './index.html' })],
});
