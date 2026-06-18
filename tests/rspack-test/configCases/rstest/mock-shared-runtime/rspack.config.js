const path = require('path');
const {
  experiments: { RstestPlugin },
} = require('@rspack/core');

class RstestSimpleRuntimePlugin {
  apply(compiler) {
    const { RuntimeModule } = compiler.rspack;
    class RstestRuntimeModule extends RuntimeModule {
      constructor() {
        super('rstest runtime');
      }

      generate() {
        return `
const originalRequire = __webpack_require__;
__webpack_require__ = function(...args) {
  return originalRequire(...args);
};

Object.keys(originalRequire).forEach(key => {
  __webpack_require__[key] = originalRequire[key];
});

__webpack_require__.rstest_original_modules = {};
__webpack_require__.rstest_original_module_factories = {};

__webpack_require__.rstest_mock = (id, modFactory) => {
  __webpack_require__.rstest_original_modules[id] = __webpack_require__(id);
  __webpack_require__.rstest_original_module_factories[id] = __webpack_modules__[id];

  __webpack_modules__[id] = function(
    __unused_webpack_module,
    __webpack_exports__,
    __webpack_require__,
  ) {
    __webpack_require__.r(__webpack_exports__);
    const res = modFactory();
    for (const key in res) {
      __webpack_require__.d(__webpack_exports__, {
        [key]: () => res[key],
      });
    }
  };

  delete __webpack_module_cache__[id];
};
`;
      }
    }

    compiler.hooks.thisCompilation.tap(
      'RstestSimpleRuntimePlugin',
      (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'RstestSimpleRuntimePlugin',
          (chunk) => {
            compilation.addRuntimeModule(chunk, new RstestRuntimeModule());
          },
        );
      },
    );
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = [
  {
    entry: {
      setup: './src/setup.js',
      test: './src/test.js',
    },
    target: 'node',
    experiments: {
      outputModule: true,
    },
    output: {
      filename: '[name].mjs',
      chunkFilename: '[name].mjs',
      chunkFormat: 'module',
      module: true,
    },
    optimization: {
      usedExports: false,
      mangleExports: false,
      concatenateModules: false,
      minimize: false,
      moduleIds: 'named',
      chunkIds: 'named',
      runtimeChunk: {
        name: 'rstest-runtime',
      },
    },
    plugins: [
      new RstestSimpleRuntimePlugin(),
      new RstestPlugin({
        injectModulePathName: false,
        importMetaPathName: false,
        hoistMockModule: false,
        manualMockRoot: path.resolve(__dirname, '__mocks__'),
      }),
    ],
  },
  {
    entry: {
      main: './index.js',
    },
    target: 'node',
    output: {
      filename: '[name].js',
    },
    externalsPresets: {
      node: true,
    },
  },
];
