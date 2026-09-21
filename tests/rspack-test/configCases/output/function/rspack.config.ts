import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry() {
    return {
      a: './a',
      b: './b',
    };
  },
  output: {
    filename: (data) => {
      assert(data.chunk);
      return data.chunk.name === 'a' ? `${data.chunk.name}.js` : '[name].js';
    },
  },
});
