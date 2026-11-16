import {
  rspack,
  type ExternalItemObjectUnknown,
  type RspackOptions,
} from '@rspack/core';

export default (['umd', 'umd2'] as const).flatMap((type) =>
  [false, true].map((strict): RspackOptions => {
    const externals: ExternalItemObjectUnknown = {};
    for (const branch of ['root', 'commonjs', 'commonjs2', 'amd', 'all']) {
      const target = Object.fromEntries(
        ['root', 'commonjs', 'commonjs2', 'amd'].map((key) => [
          key,
          branch === key || branch === 'all' ? ['dep', 'nested'] : ['dep'],
        ]),
      );
      externals[`${branch}-plain`] = target;
      externals[`${branch}-esm`] = { ...target, interop: 'esModule' };
      externals[`${branch}-default`] = { ...target, interop: 'default' };
    }
    return {
      target: 'node',
      output: { library: { type }, globalObject: 'globalThis' },
      externalsType: type,
      externals,
      module: {
        rules: [
          {
            test: /\.js$/,
            type: strict ? 'javascript/esm' : 'javascript/auto',
          },
        ],
      },
      plugins: [
        new rspack.DefinePlugin({ STRICT_IMPORTER: JSON.stringify(strict) }),
      ],
    };
  }),
);
