import { defineConfig } from '@rspack/cli';
import { type Chunk, type PathData } from '@rspack/core';

function checkRealChunk(data: PathData, name: string | undefined) {
  const chunk = data.chunk as Chunk;
  expect(data.chunk).toBeTruthy();
  expect(chunk.name).toBe(name);
  expect(typeof chunk.getEntryOptions).toBe('function');
  expect(typeof chunk.canBeInitial).toBe('function');
  return chunk;
}

export default defineConfig({
  mode: 'development',
  entry: {
    a: './a',
    b: {
      import: './b',
      filename: (data) => {
        const name = checkRealChunk(data, 'b').name!;
        return name + name + name + '.js';
      },
    },
  },
  output: {
    filename: (data) => {
      const name = checkRealChunk(data, data.chunk?.name).name!;
      return name + name + '.js';
    },
    chunkFilename: (data) => {
      const name = checkRealChunk(data, data.chunk?.name).name!;
      return name + name + '.js';
    },
  },
});
