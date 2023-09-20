module.exports = [
	{
		entry: "./a.js",
		output: {
			filename: "a.js",
			uniqueName: "a"
		}
	},
	{
		entry: "./b.js",
		output: {
			filename: "b.js",
			uniqueName: "b"
		}
	},
	{
		entry: "./index.js",
		output: {
			filename: "index.js"
		}
	}
];
