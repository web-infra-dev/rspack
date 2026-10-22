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
      return data.chunk?.name === 'a' ? `${data.chunk?.name}.js` : '[name].js';
    },
  },
});
