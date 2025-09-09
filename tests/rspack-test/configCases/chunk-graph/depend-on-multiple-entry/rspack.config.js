module.exports = {
	entry: {
		vendor1: "./vendor1.js",
		vendor2: "./vendor2.js",
		main: {
			dependOn: ["vendor1", "vendor2"],
			import: "./main.js"
		}
	},
	target: "web",
	node: {
		__dirname: false
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		moduleIds: "named",
		chunkIds: "named"
	}
};
