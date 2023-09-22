const path = require("path");
const { validate: validateOptions } = require("schema-utils");

const reactRefreshPath = require.resolve("../client/reactRefresh.js");
const reactRefreshEntryPath = require.resolve("../client/reactRefreshEntry.js");
const schema = require("./options.json");
const { normalizeOptions } = require("./options");

const refreshUtilsPath = require.resolve(
	"@pmmmwh/react-refresh-webpack-plugin/lib/runtime/RefreshUtils",
	{
		paths: [reactRefreshPath]
	}
);
const refreshRuntimeDirPath = path.dirname(
	require.resolve("react-refresh", {
		paths: [reactRefreshPath]
	})
);
const runtimePaths = [
	reactRefreshPath,
	refreshUtilsPath,
	reactRefreshEntryPath,
	refreshRuntimeDirPath
];

/**
 * @typedef {Object} Options
 * @property {(string | RegExp | (string | RegExp)[] | null)=} include included resourcePath for loader
 * @property {(string | RegExp | (string | RegExp)[] | null)=} exclude excluded resourcePath for loader
 */

module.exports = class ReactRefreshRspackPlugin {
	/**
	 * @param {Options} options
	 */
	constructor(options = {}) {
		validateOptions(schema, options, {
			name: "React Refresh Rspack Plugin",
			baseDataPath: "options"
		});
		/**
		 * @type {Options}
		 */
		this.options = normalizeOptions(options);
	}
	apply(compiler) {
		new compiler.webpack.EntryPlugin(compiler.context, reactRefreshEntryPath, {
			name: undefined
		}).apply(compiler);
		new compiler.webpack.ProvidePlugin({
			$ReactRefreshRuntime$: reactRefreshPath
		}).apply(compiler);

		compiler.options.module.rules.unshift({
			include: this.options.include,
			exclude: {
				or: [this.options.exclude, [...runtimePaths]].filter(Boolean)
			},
			use: "builtin:react-refresh-loader"
		});
	}
};
