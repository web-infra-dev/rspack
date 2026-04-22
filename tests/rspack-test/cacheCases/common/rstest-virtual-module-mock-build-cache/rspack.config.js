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
if (typeof __webpack_require__ === 'undefined') {
  return;
}

const originalRequire = __webpack_require__;
__webpack_require__ = function(...args) {
  try {
    return originalRequire(...args);
  } catch (e) {
    const errMsg = e.message ?? e.toString();
    if (errMsg.includes('__webpack_modules__[moduleId] is not a function')) {
      throw new Error(\`Cannot find module '\${args[0]}'\`);
    }
    throw e;
  }
};

Object.keys(originalRequire).forEach(key => {
  __webpack_require__[key] = originalRequire[key];
});

__webpack_require__.rstest_original_modules = {};

__webpack_require__.rstest_mock = (id, modFactory) => {
  let requiredModule = undefined;
  try {
    requiredModule = __webpack_require__(id);
  } catch {}
  finally {
    __webpack_require__.rstest_original_modules[id] = requiredModule;
  }

  if (typeof modFactory === 'string' || typeof modFactory === 'number') {
    __webpack_module_cache__[id] = { exports: __webpack_require__(modFactory) };
  } else if (typeof modFactory === 'function') {
    const finalModFactory = function(
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

    __webpack_modules__[id] = finalModFactory;
    delete __webpack_module_cache__[id];
  }
};

__webpack_require__.rstest_hoisted = fn => fn();
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

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  target: 'node',
  node: {
    __filename: false,
    __dirname: false,
  },
  cache: {
    type: 'persistent',
  },
  optimization: {
    usedExports: false,
    mangleExports: false,
    concatenateModules: false,
    moduleIds: 'named',
  },
  output: {
    library: { type: 'commonjs2' },
  },
  externals: {
    'virtual-module': 'node-commonjs virtual-module1',
  },
  plugins: [
    new RstestSimpleRuntimePlugin(),
    new RstestPlugin({
      injectModulePathName: true,
      hoistMockModule: true,
      importMetaPathName: true,
      manualMockRoot: path.resolve(__dirname, '__mocks__'),
      globals: true,
    }),
  ],
};
