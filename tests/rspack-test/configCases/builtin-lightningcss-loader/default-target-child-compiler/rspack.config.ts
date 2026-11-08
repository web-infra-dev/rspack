import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: ['web', 'browserslist:chrome > 95'],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        use: [
          {
            loader: 'builtin:lightningcss-loader',
            // An options object without `targets`: the targets are derived from
            // `target`, and the object is shared with the child compiler below.
            options: {},
          },
        ],
      },
    ],
  },
  node: {
    __dirname: false,
  },
  plugins: [
    // Runs a child compiler with the same module rules, like
    // html-webpack-plugin does.
    definePlugin(function (compiler) {
      compiler.hooks.make.tapAsync('test', (compilation, callback) => {
        const child = compilation.createChildCompiler(
          'test',
          {
            filename: '__child-[name].js',
            publicPath: '',
          },
          [
            new compiler.rspack.EntryPlugin(
              compilation.options.context!,
              './child-entry.js',
              { name: 'main' },
            ),
          ],
        );
        child.runAsChild((error) => callback(error));
      });
    }),
  ],
});
