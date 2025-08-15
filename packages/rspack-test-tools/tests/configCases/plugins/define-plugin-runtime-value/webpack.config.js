const fs = require("fs");
const path = require("path");
const DefinePlugin = require("@rspack/core").DefinePlugin;

const valueFile = path.join(__dirname, "test-value.txt");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new DefinePlugin({
			// Test basic runtimeValue with file dependency
			TEST_VALUE1: DefinePlugin.runtimeValue(() => {
				return JSON.stringify(fs.readFileSync(valueFile, "utf-8").trim());
			}, [valueFile]),

			// Test runtimeValue with empty dependencies
			TEST_VALUE2: DefinePlugin.runtimeValue(() => {
				return JSON.stringify("static-runtime-value");
			}, []),

			// Test runtimeValue with true (uncacheable)
			TEST_VALUE3: DefinePlugin.runtimeValue(() => {
				return JSON.stringify(JSON.stringify(Date.now()));
			}, true),

			// Test runtimeValue with options object
			TEST_VALUE4: DefinePlugin.runtimeValue(
				() => {
					return JSON.stringify(
						fs.readFileSync(valueFile, "utf-8").trim() + "-with-options"
					);
				},
				{
					fileDependencies: [valueFile]
				}
			),

			// Test runtimeValue with version function
			TEST_VALUE5: DefinePlugin.runtimeValue(
				({ version, key }) => {
					return JSON.stringify(JSON.stringify({ version, key }));
				},
				{
					version: () => "1.0.0"
				}
			),

			// Test nested object with runtimeValue
			NESTED: {
				VALUE: DefinePlugin.runtimeValue(() => {
					return JSON.stringify("nested-value");
				}, [])
			},

			// Test runtimeValue that returns different types
			RUNTIME_NUMBER: DefinePlugin.runtimeValue(() => 42, []),
			RUNTIME_BOOLEAN: DefinePlugin.runtimeValue(() => true, []),
			RUNTIME_NULL: DefinePlugin.runtimeValue(() => null, []),
			RUNTIME_UNDEFINED: DefinePlugin.runtimeValue(() => undefined, []),

			// Test error handling
			ERROR_VALUE: DefinePlugin.runtimeValue(() => {
				throw new Error("Test error");
			}, []),

			// Test with module context
			MODULE_VALUE: DefinePlugin.runtimeValue(({ module }) => {
				return JSON.stringify(module ? "has-module" : "no-module");
			}, [])
		})
	]
};
