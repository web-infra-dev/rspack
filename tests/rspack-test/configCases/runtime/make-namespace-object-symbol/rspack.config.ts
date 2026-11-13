import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const cases = [
  { symbol: false, computedProperty: false, shape: 'feature-test' },
  { symbol: true, computedProperty: false, shape: 'define-property' },
  { symbol: true, computedProperty: true, shape: 'define-properties' },
];

export default cases.map(({ symbol, computedProperty, shape }, i) =>
  defineConfig({
    output: {
      filename: `bundle${i}.js`,
      environment: {
        symbol,
        computedProperty,
      },
    },
    plugins: [
      new rspack.DefinePlugin({
        SHAPE: JSON.stringify(shape),
      }),
    ],
  }),
);
