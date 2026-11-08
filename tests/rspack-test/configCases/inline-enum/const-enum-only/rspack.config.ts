import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
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
                exportedEnum: 'const-only',
              },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    moduleIds: 'named',
    concatenateModules: false,
    inlineExports: true,
  },
});
