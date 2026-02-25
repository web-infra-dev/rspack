const path = require("path");
const fs = require("fs");
const { rspack } = require("@rspack/core");
/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { srcPath }) => {
	const valueFile = path.resolve(srcPath, "value.txt");
	return {
		plugins: [
			new rspack.DefinePlugin({
				TEST_VALUE: rspack.DefinePlugin.runtimeValue(() => {
					return JSON.stringify(fs.readFileSync(valueFile, "utf-8").trim());
				}, [valueFile]),
				TEST_VALUE2: rspack.DefinePlugin.runtimeValue(() => {
					return JSON.stringify(fs.readFileSync(valueFile, "utf-8").trim());
				}, []),
				TEST_VALUE3: rspack.DefinePlugin.runtimeValue(() => {
					return JSON.stringify(fs.readFileSync(valueFile, "utf-8").trim());
				}, true),
				TEST_VALUE4: rspack.DefinePlugin.runtimeValue(
					() => {
						return JSON.stringify(fs.readFileSync(valueFile, "utf-8").trim());
					},
					{
						fileDependencies: [valueFile]
					}
				),
				TEST_VALUE5: rspack.DefinePlugin.runtimeValue(
					({ version, key }) => {
						return JSON.stringify({ version, key });
					},
					{
						version: () => fs.readFileSync(valueFile, "utf-8").trim()
					}
				)
			})
		]
	};
};
