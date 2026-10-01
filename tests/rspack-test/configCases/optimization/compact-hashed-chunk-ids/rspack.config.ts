import { defineConfig } from '@rspack/cli';
import { type Compiler, rspack } from '@rspack/core';

if (
  rspack.ids.CompatHashedChunkIdsPlugin !==
  rspack.ids.CompactHashedChunkIdsPlugin
) {
  throw new Error(
    'CompatHashedChunkIdsPlugin must alias CompactHashedChunkIdsPlugin',
  );
}

const checkChunkIds =
  (minLength: number, expectExtended = false) =>
  (compiler: Compiler) => {
    compiler.hooks.done.tap('CheckCompactHashedChunkIds', (stats) => {
      const ids = stats
        .toJson({
          all: false,
          chunks: true,
          ids: true,
        })
        .chunks!.map((chunk) => String(chunk.id));

      expect(new Set(ids).size).toBe(ids.length);
      for (const id of ids) {
        expect(id).toMatch(/^[a-z0-9]+$/);
        expect(id.length).toBeGreaterThanOrEqual(minLength);
      }
      if (expectExtended) {
        expect(ids.some((id) => id.length === minLength)).toBe(true);
        expect(ids.some((id) => id.length > minLength)).toBe(true);
      }
    });
  };

export default defineConfig([
  {
    optimization: {
      chunkIds: 'compact-hashed',
    },
    plugins: [checkChunkIds(1)],
  },
  {
    entry: './min-length',
    optimization: {
      chunkIds: false,
    },
    plugins: [
      new rspack.ids.CompactHashedChunkIdsPlugin({ minLength: 1 }),
      checkChunkIds(1, true),
    ],
  },
  {
    optimization: {
      chunkIds: 'compat-hashed',
    },
    plugins: [checkChunkIds(1)],
  },
]);
