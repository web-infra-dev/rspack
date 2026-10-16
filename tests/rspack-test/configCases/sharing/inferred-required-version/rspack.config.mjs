import { container, sharing } from '@rspack/core';
import cases from './cases.mjs';

const shared = Object.fromEntries(
  cases.map(({ name, singleton = false, requiredVersion }) => [
    name,
    { import: false, strictVersion: true, singleton, requiredVersion },
  ]),
);

export default [
  {
    plugins: [new sharing.ConsumeSharedPlugin({ consumes: shared })],
  },
  {
    plugins: [
      new container.ModuleFederationPlugin({
        name: 'inferred_required_version',
        shared,
        runtimePlugins: ['./runtime-plugin.mjs'],
      }),
    ],
  },
];
