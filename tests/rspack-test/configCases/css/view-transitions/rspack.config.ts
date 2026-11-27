import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        parser: { pure: true },
        generator: { localIdentName: 'demo-[local]' },
      },
    ],
  },
});
