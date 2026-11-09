import { defineConfig } from '@rspack/cli';

const parserRule = {
  test: /\.toml$/,
  parser: {
    parse: () => ({ foo: 'bar' }),
  },
};
const typeRule = { test: /\.toml$/, type: 'json' };

export default defineConfig([
  { module: { rules: [{ ...parserRule, ...typeRule }] } },
  {
    module: {
      rules: [parserRule, { ...typeRule, parser: { exportsDepth: 1 } }],
    },
  },
]);
