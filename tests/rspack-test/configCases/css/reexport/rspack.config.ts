import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['development', 'production'] as const).map((mode, idx) =>
    defineConfig({
      externals: {
        fs: 'node-commonjs fs',
        path: 'node-commonjs path',
      },
      name: mode,
      devtool: false,
      entry: './index.js',
      mode,
      target: 'web',
      output: {
        filename: `bundle${idx}.js`,
      },
      node: {
        __dirname: false,
        __filename: false,
      },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'css/auto',
          },
        ],
      },
      optimization: {
        minimize: false,
      },
      experiments: {
        css: true,
      },
    }),
  ),
);
