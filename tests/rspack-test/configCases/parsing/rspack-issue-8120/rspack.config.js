const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.DefinePlugin({
			"process.env.test": {
				NODE_ENV: '"development"',
				PUBLIC_URL: '""',
				WDS_SOCKET_HOST: undefined,
				WDS_SOCKET_PATH: undefined,
				WDS_SOCKET_PORT: undefined,
				FAST_REFRESH: "true"
			}
		})
	]
};
