it("should be able to use require.js-style define", function(done) {
	const template = "a"
	define("name", ["./templates/" + template, true ? "./circular" : "fail"], function(templateA, circular) {
		expect(templateA).toBe("a");
		expect(circular).toBe(1);
		done();
	});
});
