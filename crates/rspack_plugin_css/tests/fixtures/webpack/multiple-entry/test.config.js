module.exports = {
	mode: "development",
	entry: {
		"main-one": {
			import: ["./index-one.js"]
		},
		"main-two": {
			import: ["./index-two.js"]
		}
	}
};
