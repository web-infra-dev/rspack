const { rspack } = require('@rspack/core');
const path = require('path');

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
  {
    externals: {
      './249.bundle1.js': 'commonjs ./249.bundle1.js',
      './use-style_js.bundle0.js': 'commonjs ./use-style_js.bundle0.js',
    },
    target: 'web',
    mode: 'development',
    module: {
      generator: {
        'css/auto': {
          localIdentName: '[path][name][ext]-[local]',
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    externals: {
      './249.bundle1.js': 'commonjs ./249.bundle1.js',
      './use-style_js.bundle0.js': 'commonjs ./use-style_js.bundle0.js',
    },
    target: 'web',
    mode: 'production',
    output: {
      uniqueName: 'my-app',
    },
    incremental: {
      moduleIds: false,
    },
    optimization: {
      concatenateModules: false,
      moduleIds: false,
    },
    ignoreWarnings: [
      /Inconsistent rule global\/local/,
      /A ':global\(' is not allowed inside of a ':local\(\)' or ':global\(\)'/,
      /A ':local\(' is not allowed inside of a ':local\(\)' or ':global\(\)'/,
    ],
    module: {
      generator: {
        'css/auto': {
          localIdentName: '[path][name][ext]-[local]',
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
    plugins: [
      new rspack.ids.DeterministicModuleIdsPlugin({
        maxLength: 3,
        failOnConflict: true,
        fixedLength: true,
      }),
      new rspack.experiments.ids.SyncModuleIdsPlugin({
        path: path.resolve(testPath, 'module-ids.json'),
        mode: 'create',
      }),
    ],
  },
];
