import { defineConfig } from '@rspack/cli';
import type { RuleSetRule } from '@rspack/core';

export default defineConfig(
  [false, true].flatMap((useFunction) =>
    [false, true].map((reverse) => {
      const condition = (pattern: RegExp) =>
        useFunction ? (value: string) => pattern.test(value) : pattern;
      const equalPriorityRules: RuleSetRule[] = [
        { priority: 2, type: 'asset/resource' },
        { priority: 2, generator: { filename: 'equal/[name][ext]' } },
      ];
      const rules: RuleSetRule[] = [
        {
          test: condition(/\.ts$/),
          type: 'javascript/auto',
          use: {
            loader: 'builtin:swc-loader',
            options: { jsc: { parser: { syntax: 'typescript' } } },
          },
        },
        {
          test: condition(/\.ts$/),
          resourceQuery: condition(/url/),
          priority: 1,
          type: 'asset/resource',
          generator: { filename: 'url/[name][ext]' },
        },
        {
          test: condition(/\.txt$/),
          resourceQuery: condition(/equal/),
          // The adapter needs the parent type to serialize a child's generator.
          // The parent's priority stays at 0, so only the two children survive.
          type: 'asset/resource',
          rules: reverse ? equalPriorityRules.reverse() : equalPriorityRules,
        },
        {
          test: condition(/\.txt$/),
          resourceQuery: condition(/equal/),
          priority: 1,
          enforce: 'pre',
          loader: './reject-loader.mjs',
        },
        {
          test: condition(/\.txt$/),
          resourceQuery: condition(/equal/),
          enforce: 'post',
          loader: './reject-loader.mjs',
        },
        {
          test: condition(/\.txt$/),
          resourceQuery: condition(/default/),
          type: 'asset/source',
        },
        {
          test: condition(/\.txt$/),
          resourceQuery: condition(/default/),
          priority: 0,
          loader: './append-loader.mjs',
        },
        {
          test: condition(/never-matches$/),
          priority: 100,
          loader: './reject-loader.mjs',
        },
      ];
      return {
        output: { publicPath: '' },
        module: { rules: reverse ? rules.reverse() : rules },
      };
    }),
  ),
);
