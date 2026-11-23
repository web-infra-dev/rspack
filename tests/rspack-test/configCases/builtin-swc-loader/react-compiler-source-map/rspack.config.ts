import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  module: {
    rules: [
      {
        test: /\.jsx$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            detectSyntax: 'auto',
            jsc: {
              transform: {
                reactCompiler: true,
                react: { runtime: 'automatic' },
              },
            },
          },
        },
        type: 'javascript/auto',
      },
    ],
  },
});
