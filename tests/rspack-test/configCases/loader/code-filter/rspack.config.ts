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

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      filtered(/[\\/]string\.js$/, 'marker:string', 'string'),
      filtered(/regexp-[ab]\.js$/, /marker:regexp/g, 'regexp'),
      filtered(/array\.js$/, ['not present', /marker:array/], 'array'),
      filtered(
        /excluded\.js$/,
        { include: 'marker:include', exclude: /marker:exclude/ },
        'excluded',
      ),
      filtered(/empty-include\.js$/, { include: [] }, 'empty-include'),
      filtered(/empty-exclude\.js$/, { exclude: [] }, 'empty-exclude'),
      filtered(/empty-string\.js$/, '', 'empty-string'),
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
            filter: { code: 'marker:functional' },
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
            filter: { code: /generated:marker/g },
          },
          './marker-producer-loader.mjs',
        ],
      },
      {
        test: /metadata\.js$/,
        use: [
          './metadata-observer-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'should-not-run' },
            filter: { code: 'not present' },
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
            filter: { code: '-simple' },
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
            loader: './transform-loader.mjs',
            options: { value: 'should-not-run' },
            filter: { code: '' },
          },
          './invalid-producer-loader.mjs',
        ],
      },
      {
        test: /native-hit\.txt$/,
        use: [
          {
            loader: 'builtin:test-simple-loader',
            filter: { code: 'marker' },
          },
        ],
      },
      {
        test: /native-miss\.js$/,
        use: [
          {
            loader: 'builtin:test-simple-loader',
            filter: { code: 'not present' },
          },
        ],
      },
      {
        test: /pitch-short-circuit\.js$/,
        use: [
          {
            loader: './pitch-result-loader.mjs',
            filter: { code: 'not present' },
          },
        ],
      },
    ],
  },
});
