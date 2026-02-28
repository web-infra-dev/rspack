module.exports = [
	{
		entry: "./index.js",
		module: {
			noParse: require.resolve("./not-parsed-a")
		}
	},
	{
		entry: "./index.js",
		module: {
			noParse: /not-parsed/
		}
	},
	{
		entry: "./index.js",
		module: {
			noParse(content) {
				return /not-parsed/.test(content);
			}
		}
	},
	{
		entry: "./index.js",
		module: {
			noParse: /not-parsed/,
			rules: [{ test: /\.js$/, loader: "builtin:swc-loader" }]
		}
	}
];
