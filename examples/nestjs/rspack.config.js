const { RunScriptWebpackPlugin } = require("run-script-webpack-plugin");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	target: "node",
	entry: {
		main: ["webpack/hot/poll?100", "./src/main.ts"]
	},
	optimization: {
		minimize: false
	},
	externalsType: "commonjs",
	plugins: [
		!process.env.BUILD &&
			new RunScriptWebpackPlugin({
				name: "main.js",
				autoRestart: false
			})
	].filter(Boolean),
	devServer: {
		devMiddleware: {
			writeToDisk: true
		}
	},
	externals: [
		function (obj, callback) {
			const resource = obj.request;
			const lazyImports = [
				"@nestjs/core",
				"@nestjs/microservices",
				"@nestjs/platform-express",
				"cache-manager",
				"class-validator",
				"class-transformer",
				// ADD THIS
				"@nestjs/microservices/microservices-module",
				"@nestjs/websockets",
				"socket.io-adapter",
				"utf-8-validate",
				"bufferutil",
				"kerberos",
				"@mongodb-js/zstd",
				"snappy",
				"@aws-sdk/credential-providers",
				"mongodb-client-encryption",
				"@nestjs/websockets/socket-module",
				"bson-ext",
				"snappy/package.json",
				"aws4"
			];
			if (!lazyImports.includes(resource)) {
				return callback();
			}
			try {
				require.resolve(resource);
			} catch (err) {
				callback(null, resource);
			}
			callback();
		}
	]
};
module.exports = config;
