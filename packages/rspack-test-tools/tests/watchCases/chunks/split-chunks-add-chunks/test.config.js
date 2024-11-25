module.exports = {
	findBundle(i, options, step) {
		if (step === "2") {
			return ["lib1.js", "lib2.js", "main.js"];
		}
		return ["lib1.js", "main.js"];
	}
};
