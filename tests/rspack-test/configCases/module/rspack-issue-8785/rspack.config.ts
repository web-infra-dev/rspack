import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.toml$/,
        parser: {
          parse: () => ({ foo: 'bar' }),
        },
      },
      {
        test: /\.toml$/,
        type: 'json',
      },
    ],
  },
});
