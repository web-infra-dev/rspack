module.exports = {
	findBundle: function () {
		return [
			// DIFF: if the entry is ignored, webpack will generate an empty module which only contains a comment in it
			// "./a.js",
			// "./b.js",
			"./bundle0.js"
		]
	}
};
