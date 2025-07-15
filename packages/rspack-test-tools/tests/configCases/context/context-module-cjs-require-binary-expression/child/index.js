it("context module(sync) + cjs require + binary expression", function (done) {
	const a = "child/index";
	const module = require("./" + a + ".js");
	expect(module.value).toBe("dynamic");
	done();
});
