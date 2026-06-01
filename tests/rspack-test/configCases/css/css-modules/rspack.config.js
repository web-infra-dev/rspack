'use strict';

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
  {
    target: 'web',
    mode: 'development',

    module: {
      rules: [
        {
          test: /\.module\.css$/i,
          type: 'css/auto',
        },
        {
          test: /\.my-css$/i,
          type: 'css/auto',
        },
        {
          test: /\.invalid$/i,
          type: 'css/auto',
        },
      ],
    },
    node: {
      __dirname: false,
      __filename: false,
    },
  },
  // TODO: enable the production compiler once Rspack supports the remaining
  // webpack css-modules fixture behavior covered by this case.
  // {
  //   target: 'web',
  //   mode: 'production',
  //   output: {
  //     uniqueName: 'my-app',
  //   },
  //   module: {
  //     rules: [
  //       {
  //         test: /\.my-css$/i,
  //         type: 'css/auto',
  //       },
  //       {
  //         test: /\.invalid$/i,
  //         type: 'css/auto',
  //       },
  //     ],
  //   },
  //   node: {
  //     __dirname: false,
  //     __filename: false,
  //   },
  //   plugins: [
  //     new rspack.ids.DeterministicModuleIdsPlugin({
  //       maxLength: 3,
  //       failOnConflict: true,
  //       fixedLength: true,
  //       test: (m) => m.type.startsWith('css'),
  //     }),
  //     new rspack.experiments.ids.SyncModuleIdsPlugin({
  //       test: (m) => m.type.startsWith('css'),
  //       path: path.resolve(testPath, 'module-ids.json'),
  //       mode: 'create',
  //     }),
  //   ],
  // },
];
