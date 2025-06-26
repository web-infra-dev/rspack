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

if (typeof __webpack_module_cache__ !== 'undefined') {
  __webpack_require__.c = __webpack_module_cache__;
}

__webpack_require__.mocked_modules = {};

const unifyNodeProtocol = (id) => {
  if (id.startsWith('node:')) {
    return id.slice(5);
  }
  return id;
};

__webpack_require__.set_mock = (id, modFactory) => {
  if (typeof modFactory !== 'function') {
    const mockFromId = modFactory;
    const mockToId = id;
    __webpack_require__.c[mockFromId] = { exports: __webpack_require__(mockToId) };
  } else {
    __webpack_require__.c[id] = { exports: modFactory() };
  }

};
__webpack_require__.get_mock = (id) => {
  let currentMock = __webpack_require__.mocked_modules[id];
  if (currentMock) {
    return currentMock;
  }
};
__webpack_require__.rstest_require = (...args) => {
  let currentMock = __webpack_require__.mocked_modules[args[0]];
  if (currentMock) {
    const bypassedId = currentMock.qqq
    const raw = __webpack_require__.mocked_modules[bypassedId]
    if(raw) {
      delete __webpack_require__.mocked_modules[bypassedId]
    }
    const res = currentMock();
    if(raw) {
      __webpack_require__.mocked_modules[bypassedId] = raw
    }
    return res;
  }
  return __webpack_require__(...args)
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

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: "./index.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		optimization: {
			mangleExports: false,
			concatenateModules: false
		},
		plugins: [
			new RstestSimpleRuntimePlugin(),
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: path.resolve(__dirname, "__mocks__")
			})
		]
	},
	{
		entry: "./manual.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		optimization: {
			// TODO: should only mark mocked modules as used.
			usedExports: false,
			mangleExports: false,
			concatenateModules: false
		},
		plugins: [
			new RstestSimpleRuntimePlugin(),
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: path.resolve(__dirname, "__mocks__")
			})
		]
	},
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
	}
];
