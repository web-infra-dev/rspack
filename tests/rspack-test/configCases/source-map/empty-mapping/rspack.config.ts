import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'web',
  devtool: 'source-map',
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        use: ['./loader.mjs', 'builtin:lightningcss-loader'],
        sideEffects: true,
        type: 'css/auto',
      },
    ],
  },
});
