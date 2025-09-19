module.exports = {
	documentType: "jsdom",
	findBundle: function () {
		return [
			"./inner1/inner2/a.js",
			"./b.js"
		];
	}
};
