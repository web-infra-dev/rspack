import { defineConfig } from '@rspack/cli';
import type { RuleSetLoaderCodeFilter } from '@rspack/core';

const filtered = (
  test: RegExp,
  code: RuleSetLoaderCodeFilter,
  value: string,
) => ({
  test,
  use: [
    './pitch-observer-loader.mjs',
    {
      loader: './transform-loader.mjs',
      options: { value },
      filter: { code },
    },
  ],
});

const bomFiltered = (
  query: string,
  code: RuleSetLoaderCodeFilter,
  value: string,
  rustBoundary: boolean,
) => ({
  resourceQuery: new RegExp(`^\\?${query}$`),
  use: [
    {
      loader: './transform-loader.mjs',
      options: { value },
      filter: { code },
    },
    ...(rustBoundary ? ['builtin:test-passthrough-loader'] : []),
    './bom-producer-loader.mjs',
  ],
});

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      bomFiltered('bom-js-hit', /^export/, 'bom-js-hit', false),
      bomFiltered('bom-rust-hit', /^export/, 'bom-rust-hit', true),
      bomFiltered('bom-js-miss', /^\uFEFFexport/, 'should-not-run', false),
      bomFiltered('bom-rust-miss', /^\uFEFFexport/, 'should-not-run', true),
      {
        test: /shared-issuer\.js$/,
        use: ({ issuer }) => [
          {
            loader: './transform-loader.mjs',
            options: { value: 'issuer-a' },
            filter: {
              code: issuer?.endsWith('issuer-a.js')
                ? '__VALUE__'
                : 'not-present',
            },
          },
        ],
      },
      filtered(/[\\/]direct\.js$/, /marker:direct/, 'direct'),
      filtered(/regexp-[ab]\.js$/, /marker:regexp/g, 'regexp'),
      filtered(/include\.js$/, 'marker:include', 'include'),
      filtered(/excluded\.js$/, 'marker:not-present', 'excluded'),
      filtered(/exclude-only\.js$/, '__VALUE__', 'exclude-only'),
      filtered(/empty-regexp\.js$/, /(?:)/, 'empty-regexp'),
      {
        test: /empty-content\.js$/,
        use: [
          './pitch-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'empty-content' },
            filter: { code: '' },
          },
          './empty-producer-loader.mjs',
        ],
      },
      filtered(/unicode\.js$/, '你好', 'unicode'),
      {
        test: /functional\.js$/,
        use: () => [
          './pitch-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'functional' },
            filter: { code: /marker:functional/ },
          },
        ],
      },
      {
        test: /right-output\.js$/,
        use: [
          './pitch-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'right-output' },
            filter: { code: 'generated:marker' },
          },
          './marker-producer-loader.mjs',
        ],
      },
      {
        test: /cached-hit\.js$/,
        use: [
          './pitch-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'cached-hit' },
            filter: { code: /generated:marker/ },
            cache: true,
          },
          {
            loader: './marker-producer-loader.mjs',
            cache: true,
          },
        ],
      },
      {
        test: /cached-miss\.js$/,
        use: [
          './pitch-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'should-not-run' },
            filter: { code: /not-present/ },
            cache: true,
          },
          {
            loader: './marker-producer-loader.mjs',
            cache: true,
          },
        ],
      },
      {
        test: /metadata\.js$/,
        use: [
          './metadata-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'should-not-run' },
            filter: { code: /not present/ },
          },
          './metadata-producer-loader.mjs',
        ],
      },
      {
        test: /mixed-native\.txt$/,
        use: [
          './pitch-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'mixed-native' },
            filter: { code: /-simple/ },
          },
          'builtin:test-simple-loader',
        ],
      },
      {
        test: /invalid-utf8\.js$/,
        use: [
          './invalid-finalizer-loader.mjs',
          './pitch-observer-loader.mjs',
          {
            loader: './invalid-utf8-transform-loader.mjs',
            filter: { code: /(?:)/ },
          },
          './invalid-producer-loader.mjs',
        ],
      },
      {
        test: /native-hit\.txt$/,
        use: [
          {
            loader: 'builtin:test-simple-loader',
            filter: { code: /marker/ },
          },
        ],
      },
      {
        test: /native-miss\.js$/,
        use: [
          {
            loader: 'builtin:test-simple-loader',
            filter: { code: /not present/ },
          },
        ],
      },
      {
        test: /pitch-short-circuit\.js$/,
        use: [
          {
            loader: './pitch-result-loader.mjs',
            filter: { code: /not present/ },
          },
        ],
      },
    ],
  },
});
