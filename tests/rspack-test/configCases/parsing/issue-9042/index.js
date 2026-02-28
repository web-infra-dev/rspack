it("should not evaluate __dirname or __filename when set to false", function () {
	if (typeof __dirname !== "undefined") {
		throw new Error()
	}
	if (typeof __filename !== "undefined") {
		throw new Error()
	}
});
