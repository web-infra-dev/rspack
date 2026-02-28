module.exports = {
	findBundle(i, options, step) {
		if (step === "0") {
			return ["bundle0.js"];
		}
		if (step === "1") {
			return ["bundle0.js", "bundle1.js"];
		}
	}
};
