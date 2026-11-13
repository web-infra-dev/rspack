import { defineConfig, definePlugin } from '@rspack/cli';

let compilerIndex = 0;

export default defineConfig({
  cache: {
    type: 'persistent',
    buildDependencies: [import.meta.filename],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        // Keep all input files unchanged and reuse the same persistent cache.
        compiler.options.module.rules = [
          {
            test: /\.css$/,
            type: 'css/module',
            generator: {
              exportsConvention:
                compilerIndex++ === 0 ? 'camel-case-only' : 'as-is',
            },
          },
        ];
      },
    }),
  ],
});
