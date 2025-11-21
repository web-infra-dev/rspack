const { RuntimeModule, RuntimeGlobals } = require("@rspack/core");

class IsolateRuntimeModule extends RuntimeModule {
	constructor(chunk) {
		super("mock-isolate");
	}

	generate() {
		return `
      __webpack_require__.mock = function() {
        return someGlobalValue;
      };
    `;
	}
}

class NonIsolateRuntimeModule extends RuntimeModule {
	constructor(chunk) {
		super("mock-non-isolate");
	}

	shouldIsolate() {
		return false;
	}

	generate() {
		return `
      var someGlobalValue = "isolated";
    `;
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	mode: "development",
	devtool: false,
	optimization: {
		minimize: false,
		sideEffects: false,
		concatenateModules: false,
		usedExports: false,
		innerGraph: false,
		providedExports: false
	},
	plugins: [
		compiler => {
			compiler.hooks.thisCompilation.tap("MockRuntimePlugin", compilation => {
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"MockRuntimePlugin",
					(chunk, set) => {
						compilation.addRuntimeModule(chunk, new NonIsolateRuntimeModule());
						compilation.addRuntimeModule(chunk, new IsolateRuntimeModule());
					}
				);
			});
		}
	]
};
