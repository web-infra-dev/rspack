module.exports = {
	findBundle: function () {
		return [
			// FIXME: the entry depenedency should generate module even when it is ignored
			// "./a.js",
			// "./b.js",
			"./bundle0.js"
		]
	}
};
