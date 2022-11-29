module.exports = {
	mode: "development",
	entry: {
		index: {
			import: ["./src/index.js"]
		}
	},
	enhanced: {},
	builtins: {
		css: {
			presetEnv: ["chrome >= 40", "firefox > 10"]
		},
		html: [
			{
				template: "index.html",
				sri: "sha384",
				filename: "chunk1.html"
			},
			{
				template: "index.html",
				sri: "sha256",
				filename: "chunk2.html"
			},
			{
				template: "index.html",
				sri: "sha512",
				filename: "chunk3.html"
			}
		]
	}
};
