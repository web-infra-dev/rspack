import { defineConfig, definePlugin } from '@rspack/cli';

let compilationCount = 0;

export default defineConfig({
  output: {
    chunkFilename: '[name].js',
  },
  incremental: {
    buildChunkGraph: true,
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.thisCompilation.tap('testcase', (compilation) => {
        compilation.hooks.optimizeTree.tapPromise('testcase', async () => {
          compilationCount += 1;
          if (compilationCount !== 2) {
            return;
          }

          const { chunks: statsChunks } = compilation
            .getStats()
            .toJson({ all: false, chunks: true });
          const renderedStates = statsChunks?.map((chunk) => chunk.rendered);
          expect(renderedStates?.length).toBeGreaterThan(0);
          expect(renderedStates).toEqual(renderedStates?.map(() => false));
        });
      });
    }),
  ],
});
