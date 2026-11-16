import {
  rspack,
  type ExternalItemObjectValue,
  type RspackOptions,
} from '@rspack/core';

export default (['commonjs', 'amd', 'umd', 'umd2'] as const).flatMap((type) =>
  [false, true].flatMap((strict) =>
    [false, true].map((concatenateModules): RspackOptions => {
      const target = {
        commonjs: 'dep',
        commonjs2: 'dep',
        amd: 'dep',
        root: 'dep',
      };
      const undefinedTarget: ExternalItemObjectValue = { ...target };
      undefinedTarget.interop = undefined;
      return {
        target: 'node',
        output: {
          library: { type: type === 'commonjs' ? 'commonjs2' : type },
          globalObject: 'globalThis',
        },
        module: {
          rules: [
            {
              test: /\.js$/,
              type: strict ? 'javascript/esm' : 'javascript/auto',
            },
          ],
        },
        optimization: { concatenateModules },
        externalsType: type,
        externals: [
          {
            'esm-ext': { ...target, interop: 'esModule' },
            'cjs-ext': { ...target, interop: 'default' },
            'plain-ext': target,
            'undefined-ext': undefinedTarget,
            'unflagged-ext': {
              commonjs: 'unflagged',
              commonjs2: 'unflagged',
              amd: 'unflagged',
              root: 'unflagged',
              interop: 'esModule',
            },
          },
          ({ request }, callback) => {
            if (request === 'callback-ext') {
              callback(null, { ...target, interop: 'esModule' }, type);
            } else if (request === 'selected-ext') {
              callback(
                null,
                {
                  commonjs: ['dep', 'nested'],
                  interop: 'esModule',
                },
                'commonjs',
              );
            } else {
              callback();
            }
          },
          ({ request }) => {
            if (request === 'sync-ext')
              return { ...target, interop: 'default' };
          },
          async ({ request }) => {
            if (request === 'promise-ext')
              return { ...target, interop: 'esModule' };
          },
        ],
        plugins: [
          new rspack.DefinePlugin({ STRICT_IMPORTER: JSON.stringify(strict) }),
          ...(type === 'amd'
            ? [
                new rspack.BannerPlugin({
                  raw: true,
                  banner:
                    'function define(deps, fn) { fn(...deps.map(dep => require(dep))); }\n',
                }),
              ]
            : []),
        ],
      };
    }),
  ),
);
