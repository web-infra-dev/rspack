import { defineConfig } from '@rspack/cli';
import toml from 'toml';

export default defineConfig([
  {
    mode: 'development',
    module: {
      rules: [
        {
          test: /\.toml$/,
          type: 'json',

          parser: {
            parse(input: string) {
              // eslint-disable-next-line prefer-rest-params
              expect(arguments).toHaveLength(1);
              return toml.parse(input);
            },
          },
        },
      ],
    },
  },
]);
