import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  experiments: {
    newCache: {
      resolver: true,
      module: false,
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
    },
  },
  resolve: {
    packageMap: path.join(import.meta.dirname, 'package-map.json'),
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('ResolverCacheTest', ({ compilation }) => {
          // Cached resolutions keep the package map as a dependency.
          expect(
            compilation.fileDependencies.has(
              path.join(compiler.context, 'package-map.json'),
            ),
          ).toBe(true);
        });
      },
    }),
  ],
});
