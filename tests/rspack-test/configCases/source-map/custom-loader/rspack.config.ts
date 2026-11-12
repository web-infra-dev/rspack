import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  devtool: false,
  plugins: [
    definePlugin((compiler) => {
      new compiler.rspack.SourceMapDevToolPlugin({}).apply(compiler);
    }),
  ],
  module: {
    rules: [
      {
        loader: './loader.mjs',
      },
    ],
  },
});
