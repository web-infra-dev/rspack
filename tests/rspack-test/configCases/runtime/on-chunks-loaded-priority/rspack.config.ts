import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  output: { chunkLoading: false, library: { type: 'commonjs2' } },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.thisCompilation.tap(
        'OnChunksLoadedPriority',
        (compilation) => {
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'OnChunksLoadedPriority',
            (_chunk, set) => {
              set.add(compiler.rspack.RuntimeGlobals.onChunksLoaded);
            },
          );
        },
      );
    }),
  ],
});
