it("should not evaluate __dirname or __filename when node option is false", function() {
	if (typeof __dirname !== "undefined") {
		throw new Error()
	}
	if (typeof __filename !== "undefined") {
		throw new Error()
	}
});
