import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

const config = defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  target: 'web',
  node: false,
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.(js|mjs|cjs|jsx)$/,
        loader: path.join(import.meta.dirname, 'diy.js'),
      },
      {
        test: /\.(js|mjs|cjs|jsx)$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              transformImport: [
                {
                  libraryName: 'aaaaa',
                  libraryDirectory: 'es',
                  style: 'css',
                },
              ],
            },
          },
        ],
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.make.tap('child', (base) => {
          const child = base.createChildCompiler('child', {}, []);
          child.runAsChild(() => {});
        });
      },
    }),
  ],
});

export default config;
