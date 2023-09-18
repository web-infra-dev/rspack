const path = require("path");
const reactRefreshPath = require.resolve("../client/reactRefresh.js");
const reactRefreshEntryPath = require.resolve("../client/reactRefreshEntry.js");
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
	refreshRuntimeDirPath
];

module.exports = class ReactRefreshRspackPlugin {
	apply(compiler) {
		new compiler.webpack.EntryPlugin(compiler.context, reactRefreshEntryPath, {
			name: undefined
		}).apply(compiler);
		new compiler.webpack.ProvidePlugin({
			$ReactRefreshRuntime$: reactRefreshPath
		}).apply(compiler);

		compiler.options.module.rules.push({
			include: runtimePaths,
			type: "js"
		});
	}
};
