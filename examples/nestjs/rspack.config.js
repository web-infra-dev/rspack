/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	target: "node",
	entry: {
		main: "./src/main.ts"
	},
	externalsType: "commonjs",
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
