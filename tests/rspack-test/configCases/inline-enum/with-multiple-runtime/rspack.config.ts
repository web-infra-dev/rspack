import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './a',
    b: './b',
  },
  output: {
    filename: '[name].js',
  },
  resolve: {
    extensions: ['.ts', '...'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                target: 'esnext',
              },
              collectTypeScriptInfo: {
                exportedEnum: true,
              },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    chunkIds: 'named',
    concatenateModules: false,
    inlineExports: true,
  },
});
