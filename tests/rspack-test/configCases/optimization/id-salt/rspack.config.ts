import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  (['deterministic', 'compact-hashed'] as const).flatMap((algorithm) => {
    const deterministic = algorithm === 'deterministic';
    const ChunkIds = deterministic
      ? rspack.ids.DeterministicChunkIdsPlugin
      : rspack.ids.CompactHashedChunkIdsPlugin;
    const ModuleIds = deterministic
      ? rspack.ids.DeterministicModuleIdsPlugin
      : rspack.ids.CompactHashedModuleIdsPlugin;
    const withSalt = (chunkSalt: number, moduleSalt: number) => ({
      optimization: { chunkIds: false as const, moduleIds: false as const },
      plugins: [
        new ChunkIds({ salt: chunkSalt }),
        new ModuleIds({ salt: moduleSalt }),
      ],
    });
    return [
      { optimization: { chunkIds: algorithm, moduleIds: algorithm } },
      withSalt(deterministic ? 10 : 0, 0),
      withSalt(42, 42),
      withSalt(42, 42),
    ];
  }),
);
