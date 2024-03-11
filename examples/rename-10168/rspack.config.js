/**@type {import("@rspack/cli").Configuration}*/
module.exports = {
	mode: "production",
	entry: "./index.js",
	optimization: {
		mangleExports: false,
		minimize:false,
		moduleIds: 'named',
		chunkIds: "named",
		concatenateModules: true
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}

}
