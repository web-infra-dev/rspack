import { defineConfig } from '@rspack/cli';
import { type Configuration, rspack } from '@rspack/core';

const cases: {
  target?: Configuration['target'];
  environment?: NonNullable<Configuration['output']>['environment'];
  shape: string;
}[] = [
  {
    environment: { symbol: false, computedProperty: false },
    shape: 'feature-test',
  },
  {
    environment: { symbol: true, computedProperty: false },
    shape: 'define-property',
  },
  {
    environment: { symbol: true, computedProperty: true },
    shape: 'define-properties',
  },
  { target: 'node5', shape: 'feature-test' },
  { target: 'node6', shape: 'define-properties' },
];

export default cases.map(({ target, environment, shape }, i) =>
  defineConfig({
    target,
    output: {
      filename: `bundle${i}.js`,
      environment,
    },
    plugins: [
      new rspack.DefinePlugin({
        SHAPE: JSON.stringify(shape),
      }),
    ],
  }),
);
