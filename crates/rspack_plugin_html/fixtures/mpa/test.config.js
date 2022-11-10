module.exports = {
	entry: {
		chunk1: ["./chunk1.js"],
		chunk2: ["./chunk2.js"],
		chunk3: ["./chunk3.js"]
	},
	builtins: {
		html: [
			{
				template: "index.html",
				filename: "chunk1.html",
				chunks: ["chunk1"]
			},
			{
				template: "index.html",
				filename: "chunk2.html",
				chunks: ["chunk2"]
			},
			{
				template: "index.html",
				filename: "chunk3.html",
				chunks: ["chunk3"]
			}
		]
	}
};
