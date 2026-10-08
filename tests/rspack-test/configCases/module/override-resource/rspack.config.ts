import { defineConfig } from '@rspack/cli';
import type { RuleSetRule } from '@rspack/core';

function createRules(functionConditions: boolean): RuleSetRule[] {
  const condition = (regexp: RegExp) =>
    functionConditions ? (value: string) => regexp.test(value) : regexp;
  const prefix = (value: string) => ({
    loader: './prefix-loader.mjs',
    options: { prefix: value },
  });

  return [
    {
      test: condition(
        /(?:\.source\.js|\.hidden|\.hidden\.local|trailing\.|README|unicode\.js|single\.js)$/,
      ),
      type: 'asset/source',
      enforce: 'post',
      use: './capture-loader.mjs',
      overrideResource: { ext: '.txt' },
      oneOf: [
        { test: condition(/\.js$/), use: prefix('wrong:') },
        { test: condition(/\.txt$/), use: prefix('nested:') },
      ],
    },
    {
      resourceQuery: condition(/^\?override$/),
      overrideResource: { query: '?text#value', fragment: '#new' },
      rules: [
        {
          resourceQuery: condition(/^\?text#value$/),
          resourceFragment: condition(/^#new$/),
          use: prefix('nested-query:'),
        },
      ],
    },
    { test: condition(/\.txt$/), use: prefix('next:') },
    {
      resource: condition(/\.txt$/),
      include: condition(/\.txt$/),
      exclude: condition(/\.js$/),
      use: prefix('path:'),
    },
    {
      resourceQuery: condition(/^\?inline$/),
      resourceFragment: condition(/^#fragment$/),
      use: prefix('preserved:'),
    },
    {
      test: condition(/\.txt$/),
      resourceQuery: condition(/^\?text#value$/),
      resourceFragment: condition(/^#new$/),
      use: prefix('changed:'),
      overrideResource: { ext: '.md', query: '', fragment: '' },
    },
    {
      test: condition(/\.md$/),
      resourceQuery: condition(/^$/),
      resourceFragment: condition(/^$/),
      use: prefix('cleared:'),
    },
    {
      resourceQuery: condition(/^\?remove$/),
      overrideResource: { ext: '', query: '', fragment: '' },
    },
    {
      test: condition(/[/\\]single$/),
      resourceQuery: condition(/^$/),
      resourceFragment: condition(/^$/),
      use: prefix('removed:'),
    },
  ];
}

export default defineConfig([
  { name: 'regexp', module: { rules: createRules(false) } },
  { name: 'function', module: { rules: createRules(true) } },
]);
