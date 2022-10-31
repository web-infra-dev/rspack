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
				sri: "sha384",
				filename: "chunk1.html",
				chunks: ["chunk1"]
			},
			{
				template: "index.html",
				sri: "sha256",
				filename: "chunk2.html",
				chunks: ["chunk2"]
			},
			{
				template: "index.html",
				sri: "sha512",
				filename: "chunk3.html",
				chunks: ["chunk3"]
			}
		]
	}
};
