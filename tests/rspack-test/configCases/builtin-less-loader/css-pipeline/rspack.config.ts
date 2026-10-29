import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  node: false,
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  output: { cssFilename: 'style.css' },
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'css',
        generator: { exportsOnly: false },
        use: ['builtin:lightningcss-loader', 'builtin:less-loader'],
      },
    ],
  },
});
