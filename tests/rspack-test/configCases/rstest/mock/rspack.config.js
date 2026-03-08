const path = require("path");
const {
	experiments: { RstestPlugin }
} = require("@rspack/core");

class RstestSimpleRuntimePlugin {
	constructor() {}

	apply(compiler) {
		const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
		class RetestImportRuntimeModule extends RuntimeModule {
			constructor() {
				super("rstest runtime");
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
      throw new Error(\`Cannot find module '\${args[0]}'\`)
    }
    throw e;
  }
};

Object.keys(originalRequire).forEach(key => {
  __webpack_require__[key] = originalRequire[key];
});

__webpack_require__.rstest_original_modules = {};

__webpack_require__.rstest_reset_modules = () => {
  const mockedIds = Object.keys(__webpack_require__.rstest_original_modules)
  Object.keys(__webpack_module_cache__).forEach(id => {
    // Do not reset mocks registry.
    if (!mockedIds.includes(id)) {
      delete __webpack_module_cache__[id];
    }
  });
}

__webpack_require__.rstest_unmock = (id) => {
  delete __webpack_module_cache__[id]
}

__webpack_require__.rstest_require_actual = __webpack_require__.rstest_import_actual = (id) => {
  const originalModule = __webpack_require__.rstest_original_modules[id];
  // Use fallback module if the module is not mocked.
  const fallbackMod = __webpack_require__(id);
  return originalModule ? originalModule : fallbackMod;
}

__webpack_require__.rstest_exec = async (id, modFactory) => {
  if (__webpack_module_cache__) {
    let asyncFactory = __webpack_module_cache__[id];
    if (asyncFactory && asyncFactory.constructor.name === 'AsyncFunction') {
      await asyncFactory();
    }
  }
};

__webpack_require__.rstest_mock = (id, modFactory) => {
  let requiredModule = undefined
  try {
    requiredModule = __webpack_require__(id);
  } catch {
    // TODO: non-resolved module
  } finally {
    __webpack_require__.rstest_original_modules[id] = requiredModule;
  }
  if (typeof modFactory === 'string' || typeof modFactory === 'number') {
    __webpack_module_cache__[id] = { exports: __webpack_require__(modFactory) };
  } else if (typeof modFactory === 'function') {
          const finalModFactory = function (
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

__webpack_require__.rstest_mock_require = __webpack_require__.rstest_mock;

__webpack_require__.rstest_do_mock = (id, modFactory) => {
  let requiredModule = undefined
  try {
    requiredModule = __webpack_require__(id);
  } catch {
    // TODO: non-resolved module
  } finally {
    __webpack_require__.rstest_original_modules[id] = requiredModule;
  }
  if (typeof modFactory === 'string' || typeof modFactory === 'number') {
    __webpack_module_cache__[id] = { exports: __webpack_require__(modFactory) };
  } else if (typeof modFactory === 'function') {
    const exports = modFactory();
    __webpack_require__.r(exports);
    __webpack_module_cache__[id] = { exports, id, loaded: true };
  }
};

__webpack_require__.rstest_hoisted = (fn) => {
  return fn();
};
`;
			}
		}

		compiler.hooks.thisCompilation.tap(
			"RstestSimpleRuntimePlugin",
			compilation => {
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"RstestSimpleRuntimePlugin",
					chunk => {
						compilation.addRuntimeModule(
							chunk,
							new RetestImportRuntimeModule()
						);
					}
				);
			}
		);
	}
}

const rstestEntry = (entry, rstestPluginOptions = {}) => {
	return {
		entry,
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		optimization: {
			// TODO: should only mark mocked modules as used.
			usedExports: false,
			mangleExports: false,
			concatenateModules: false,
			moduleIds: "named"
		},
		module: {
			rules: [
				{
					test: /\.js$/,
					loader: path.resolve(__dirname, './importActualLoader.mjs'),
					with: {
						rstest: 'importActual'
					}
				}
			]
		},
		plugins: [
			new RstestSimpleRuntimePlugin(),
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: path.resolve(__dirname, "__mocks__"),
				...rstestPluginOptions,
			})
		]
	};
};

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	rstestEntry("./doMock.js"),
	rstestEntry("./mockFactory.js"),
	rstestEntry("./manualMock.js"),
	rstestEntry("./importActual.js"),
	rstestEntry("./importActualHoisted.js"),
	rstestEntry("./requireActual.js"),
	{
		entry: "./test.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		optimization: {
			mangleExports: false
		}
	},
	rstestEntry("./mockFirstArgIsImport.js"),
	rstestEntry("./globals/importActual.js"),
	rstestEntry("./globals-false/importActual.js", { globals: false }),
	{
		...rstestEntry("./hoisted.js"),
		externals: {
			"@rstest/core": "global @rstest/core"
		}
	}
];
