module.exports = {
	entry: {
		chunk1: {
			import: ["./chunk1.js"]
		},
		chunk2: {
			import: ["./chunk2.js"]
		},
		chunk3: {
			import: ["./chunk3.js"]
		}
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
