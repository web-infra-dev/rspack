import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  ['[id].js', '[id][id].js', '[id]', '[id].[fullhash].js'].flatMap(
    (chunkFilename) =>
      [false, true].map((arrowFunction) => ({
        target: 'node',
        output: {
          chunkFilename,
          environment: { arrowFunction },
        },
        optimization: { chunkIds: 'natural' },
        plugins: [
          new rspack.DefinePlugin({
            CHUNK_FILENAME: JSON.stringify(chunkFilename),
          }),
        ],
      })),
  ),
);
