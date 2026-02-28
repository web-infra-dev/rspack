module.exports = [
	{
		name: "lib",
		entry: {
			lib: "./lib.js"
		},
		output: {
			filename: "[name].js",
			library: {
				name: "lib",
				type: "umd"
			}
		},
		target: "web",
		optimization: {
			chunkIds: "named",
			moduleIds: "named",
			runtimeChunk: true
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					type: 'css/auto'
				}
			]
		}
	},
	{
		dependencies: ["lib"],
		entry: {
			main: "./index.js"
		},
		output: {
			filename: "[name].js"
		},
		target: "web",

		module: {
			rules: [
				{
					test: /\.css$/,
					type: "css/auto"
				}
			]
		}
	}
];
