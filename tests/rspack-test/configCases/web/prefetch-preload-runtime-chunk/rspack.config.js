const fs = require('fs');
const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './chunk1.js': 'commonjs ./chunk1.js',
    './chunk2.js': 'commonjs ./chunk2.js',
  },
  target: 'web',
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
    runtimeChunk: {
      name: (entrypoint) => `runtime~${entrypoint.name}`,
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', () => {
          const runtimePrefix =
            compiler.options.experiments?.runtimeMode === 'rspack'
              ? 'rspack/runtime'
              : 'webpack/runtime';
          const output = compiler.options.output.path;
          const runtime = fs.readFileSync(
            path.join(output, 'runtime~main.js'),
            'utf-8',
          );
          expect(runtime).not.toContain(
            `${runtimePrefix}/chunk_prefetch_startup`,
          );
          const main = fs.readFileSync(path.join(output, 'main.js'), 'utf-8');
          expect(main).toContain(`${runtimePrefix}/chunk_prefetch_startup`);
        });
      },
    },
  ],
};
