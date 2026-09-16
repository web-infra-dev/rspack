import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    (compiler) => {
      compiler.hooks.make.tapAsync(
        'ChildCompilerWorkerTest',
        (compilation, callback) => {
          const childCompiler = compilation.createChildCompiler(
            'child-compiler-worker-test',
            {
              filename: '__child-[name].js',
              publicPath: '',
              asyncChunks: false,
            },
            [
              new compiler.webpack.library.EnableLibraryPlugin('commonjs'),
              new compiler.webpack.EntryPlugin(
                compiler.context,
                path.join(import.meta.dirname, 'child-entry.js'),
                {
                  name: 'child-entry',
                  library: { type: 'commonjs' },
                },
              ),
            ],
          );
          childCompiler.runAsChild((err) => callback(err));
        },
      );
    },
  ],
};
