import { defineConfig, definePlugin } from '@rspack/cli';

// Every entry is a distinct module with its own `import('./shared')`, and
// duplicate chunks are not merged, so each creates an identical unnamed chunk.
const ENTRY_COUNT = 20;

export default defineConfig({
  entry: Object.fromEntries(
    Array.from({ length: ENTRY_COUNT }, (_, i) => [
      `entry${i}`,
      `./index.js?${i}`,
    ]),
  ),
  output: {
    filename: '[name].js',
    chunkFilename: '[id].js',
  },
  optimization: {
    chunkIds: 'compact-hashed',
    mergeDuplicateChunks: false,
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.done.tap('CheckDuplicateChunkIds', (stats) => {
        const asyncChunks = stats
          .toJson({ all: false, chunks: true, ids: true })
          .chunks!.filter((chunk) => chunk.names?.length === 0);
        const ids = asyncChunks.map((chunk) => String(chunk.id));

        expect(asyncChunks).toHaveLength(ENTRY_COUNT);
        expect(new Set(ids).size).toBe(ENTRY_COUNT);
        for (const id of ids) {
          expect(id.length).toBeLessThan(5);
        }
      });
    }),
  ],
});
