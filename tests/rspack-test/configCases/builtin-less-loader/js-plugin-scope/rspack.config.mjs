import assert from 'node:assert/strict';
import path from 'node:path';

export default ['builtin:less-loader', 'less-loader'].map((loader) => ({
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          { loader, options: { lessOptions: { quietDeprecations: true } } },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.afterCompile.tap(
          'LessPluginDependencies',
          (compilation) => {
            if (compilation.errors.length) return;
            assert(
              compilation.fileDependencies.has(
                path.join(import.meta.dirname, 'plugin.js'),
              ),
            );
          },
        );
      },
    },
  ],
}));
