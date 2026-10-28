import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: false,
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.json$/,
        resourceQuery: /JSONParse=false/,
        type: 'json',
        generator: { JSONParse: false },
      },
    ],
    generator: { json: { JSONParse: true } },
  },
});
