const path = require("path");
const reactRefreshPath = require.resolve("./reactRefresh.js");
const RefreshUtilsPath = require.resolve(
	"@pmmmwh/react-refresh-webpack-plugin/lib/runtime/RefreshUtils",
	{
		paths: [reactRefreshPath]
	}
);
const RefreshRuntimeDirPath = path.dirname(
	require.resolve("react-refresh", {
		paths: [reactRefreshPath]
	})
);

exports.runtimePathRegexp = new RegExp(
	`${reactRefreshPath}|${RefreshUtilsPath}|^${RefreshRuntimeDirPath}`
);
