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

__webpack_require__.before_mocked_modules = {};

__webpack_require__.reset_modules = () => {
  __webpack_module_cache__ = {};
}

__webpack_require__.unmock = (id) => {
  delete __webpack_module_cache__[id]
}

__webpack_require__.import_actual = __webpack_require__.require_actual = (id) => {
  const beforeMock = __webpack_require__.before_mocked_modules[id];
  return beforeMock;
}

__webpack_require__.set_mock = (id, modFactory) => {
  if (typeof modFactory === 'string' || typeof modFactory === 'number') {
    __webpack_require__.before_mocked_modules[id] = __webpack_require__(id);
    __webpack_module_cache__[id] = { exports: __webpack_require__(modFactory) };
  } else if (typeof modFactory === 'function') {
    __webpack_module_cache__[id] = { exports: modFactory() };
  }
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

const rstestEntry = entry => {
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
		plugins: [
			new RstestSimpleRuntimePlugin(),
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: path.resolve(__dirname, "__mocks__")
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
	}
];
