module.exports = {
	entry: {
		chunk1: "./chunk1.js",
		chunk2: "./chunk2.js",
		chunk3: "./chunk3.js"
	},
	builtins: {
		html: [
			{
				template: "index.html",
				chunks: ["chunk1", "chunk2"],
				excludedChunks: ["chunk2"]
			}
		]
	}
};
