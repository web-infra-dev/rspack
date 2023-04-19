it("should not evaluate __dirname or __filename when node option is false", function (done) {
	expect(typeof __dirname).toBe("undefined");
	expect(typeof __filename).toBe("undefined");
	// if (typeof __dirname !== "undefined") {
	// 	done.fail();
	// }
	// if (typeof __filename !== "undefined") {
	// 	done.fail();
	// }
	done();
});
